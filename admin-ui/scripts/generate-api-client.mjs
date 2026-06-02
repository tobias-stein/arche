#!/usr/bin/env node

// ============================================================
// Arche API Client Generator
//
// Fetches JSON Schema from the Arche service endpoints and
// generates TypeScript types and API client code.
//
// Usage:
//   npm run generate-api-client
//
// Environment variables:
//   ARCHE_API_URL  - Base URL of the Arche service (default: http://localhost:8080)
// ============================================================

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const SCHEMAS_DIR = join(ROOT, 'src', 'api', 'schemas');
const GENERATED_DIR = join(ROOT, 'src', 'api', 'generated');

const API_URL = process.env.ARCHE_API_URL ?? 'http://localhost:8080';

const SCHEMA_ENDPOINTS = [
  { name: 'blueprints', path: '/api/schema/blueprints' },
  { name: 'affixes', path: '/api/schema/affixes' },
  { name: 'generate', path: '/api/schema/generate' },
];

async function fetchSchema(name, url) {
  console.log(`Fetching schema from ${url}...`);
  try {
    const response = await fetch(url, {
      headers: { Accept: 'application/schema+json, application/json' },
      signal: AbortSignal.timeout(5000),
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    const body = await response.json();
    console.log(`  ✓ Fetched ${name} schema (${response.headers.get('content-type')})`);
    return body;
  } catch (err) {
    console.log(`  ✗ Failed to fetch ${name} schema: ${err.message}`);
    return null;
  }
}

function loadFallbackSchema(name) {
  const fallbackPath = join(SCHEMAS_DIR, `${name}.json`);
  if (existsSync(fallbackPath)) {
    console.log(`  → Using fallback schema for ${name}`);
    const raw = readFileSync(fallbackPath, 'utf-8');
    return JSON.parse(raw);
  }
  console.warn(`  ⚠ No fallback schema found for ${name} at ${fallbackPath}`);
  return null;
}

function saveSchema(name, data) {
  if (!existsSync(SCHEMAS_DIR)) {
    mkdirSync(SCHEMAS_DIR, { recursive: true });
  }
  const filePath = join(SCHEMAS_DIR, `${name}.json`);
  writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n', 'utf-8');
  console.log(`  ✓ Saved schema to ${filePath}`);
}

function ensureDir(dir) {
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
}

// ---- JSON Schema to TypeScript type converter ----

function schemaToTsType(schema, defs, depth = 0) {
  if (depth > 20) return 'unknown';

  if (schema.$ref) {
    const refName = schema.$ref.replace('#/$defs/', '').replace('#/definitions/', '');
    return refName;
  }

  if (schema.oneOf) {
    return schema.oneOf
      .map((s) => schemaToTsType(s, defs, depth + 1))
      .join(' | ');
  }

  if (schema.anyOf) {
    return schema.anyOf
      .map((s) => schemaToTsType(s, defs, depth + 1))
      .join(' | ');
  }

  if (schema.allOf) {
    const parts = schema.allOf.map((s) => schemaToTsType(s, defs, depth + 1));
    return parts.join(' & ');
  }

  if (schema.enum) {
    return schema.enum.map((v) => (typeof v === 'string' ? `'${v}'` : String(v))).join(' | ');
  }

  if (schema.const !== undefined) {
    return typeof schema.const === 'string' ? `'${schema.const}'` : String(schema.const);
  }

  if (schema.type === 'string') {
    if (schema.format === 'uuid') return 'string';
    if (schema.format === 'date-time') return 'string';
    return 'string';
  }

  if (schema.type === 'number' || schema.type === 'integer') return 'number';

  if (schema.type === 'boolean') return 'boolean';

  if (schema.type === 'array') {
    const itemType = schema.items ? schemaToTsType(schema.items, defs, depth + 1) : 'unknown';
    return `${itemType}[]`;
  }

  if (schema.type === 'object' || schema.properties) {
    if (!schema.properties || Object.keys(schema.properties).length === 0) {
      if (schema.additionalProperties) {
        const valType = schemaToTsType(schema.additionalProperties, defs, depth + 1);
        return `Record<string, ${valType}>`;
      }
      return 'Record<string, unknown>';
    }

    const required = new Set(schema.required ?? []);
    const props = Object.entries(schema.properties)
      .map(([key, val]) => {
        const tsType = schemaToTsType(val, defs, depth + 1);
        const isRequired = required.has(key);
        const opt = isRequired ? '' : '?';
        const safeKey = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/.test(key) ? key : `'${key}'`;
        return `  ${safeKey}${opt}: ${tsType}`;
      })
      .join(';\n');

    if (schema.additionalProperties) {
      const valType = schemaToTsType(schema.additionalProperties, defs, depth + 1);
      return `{\n${props};\n} & Record<string, ${valType}>`;
    }

    return `{\n${props};\n}`;
  }

  if (Array.isArray(schema.type)) {
    const types = schema.type.map((t) => {
      if (t === 'string') return 'string';
      if (t === 'number' || t === 'integer') return 'number';
      if (t === 'boolean') return 'boolean';
      if (t === 'null') return 'null';
      return 'unknown';
    });
    return types.join(' | ');
  }

  if (schema.type === 'null') return 'null';

  return 'unknown';
}

function generateTsInterface(name, schema, defs) {
  if (schema.enum) {
    const values = schema.enum.map((v) => (typeof v === 'string' ? `'${v}'` : String(v))).join(' | ');
    return `export type ${name} = ${values};`;
  }

  if (
    schema.oneOf ||
    schema.anyOf ||
    schema.allOf ||
    schema.$ref ||
    schema.const !== undefined
  ) {
    const tsType = schemaToTsType(schema, defs);
    if (schema.oneOf || schema.anyOf) {
      return `export type ${name} = ${tsType};`;
    }
    return `export type ${name} = ${tsType};`;
  }

  if (schema.type === 'object' || schema.properties) {
    const required = new Set(schema.required ?? []);
    const props = Object.entries(schema.properties ?? {})
      .map(([key, val]) => {
        const tsType = schemaToTsType(val, defs);
        const isRequired = required.has(key);
        const opt = isRequired ? '' : '?';
        const safeKey = /^[a-zA-Z_$][a-zA-Z0-9_$]*$/.test(key) ? key : `'${key}'`;
        return `  ${safeKey}${opt}: ${tsType};`;
      })
      .join('\n');

    return `export interface ${name} {\n${props}\n}`;
  }

  const tsType = schemaToTsType(schema, defs);
  return `export type ${name} = ${tsType};`;
}

function generateTypesFromSchema(schemaData, name) {
  const defs = schemaData.$defs ?? schemaData.definitions ?? {};
  const lines = [`// --- Types from ${name} schema ---\n`];

  for (const [defName, defSchema] of Object.entries(defs)) {
    lines.push(generateTsInterface(defName, defSchema, defs));
    lines.push('');
  }

  return lines.join('\n');
}

// ---- Main ----

async function main() {
  console.log('=== Arche API Client Generator ===\n');

  // Step 1: Fetch schemas (with fallback)
  console.log('Step 1: Fetch schemas\n');

  const schemas = {};
  for (const endpoint of SCHEMA_ENDPOINTS) {
    const url = `${API_URL}${endpoint.path}`;
    let schema = await fetchSchema(endpoint.name, url);

    if (!schema) {
      schema = loadFallbackSchema(endpoint.name);
    }

    if (schema) {
      schemas[endpoint.name] = schema;
      saveSchema(endpoint.name, schema);
    }
  }

  // Step 2: Validate schemas were fetched/loaded
  console.log('\nStep 2: Validate schemas\n');

  ensureDir(GENERATED_DIR);

  let allSchemasLoaded = true;
  for (const endpoint of SCHEMA_ENDPOINTS) {
    if (schemas[endpoint.name]) {
      console.log(`  ✓ ${endpoint.name} schema available`);
    } else {
      console.warn(`  ⚠ ${endpoint.name} schema not available`);
      allSchemasLoaded = false;
    }
  }

  if (!allSchemasLoaded) {
    console.warn('\n  ⚠ Some schemas are missing. Type accuracy may be affected.');
    console.warn('  Start the Arche service and re-run to fetch live schemas.');
  }

  // Step 3: Report completion
  console.log('\nStep 3: Generation complete');
  console.log(`  Schemas saved to: ${SCHEMAS_DIR}/`);
  console.log(`  Generated files in: ${GENERATED_DIR}/`);
  console.log('  Run `npx tsc --noEmit` to verify TypeScript compilation.\n');
}

main().catch((err) => {
  console.error('Generation failed:', err);
  process.exit(1);
});