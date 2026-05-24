import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const MINUTE = 60
const HOUR = 3600
const DAY = 86400
const WEEK = 604800
const MONTH = 2592000

export function relativeTime(iso: string): string {
  const diff = (Date.now() - new Date(iso).getTime()) / 1000
  if (diff < 0) return 'just now'
  if (diff < MINUTE) return 'just now'
  if (diff < HOUR) {
    const m = Math.floor(diff / MINUTE)
    return `${m} min ago`
  }
  if (diff < DAY) {
    const h = Math.floor(diff / HOUR)
    return `${h} hour${h > 1 ? 's' : ''} ago`
  }
  if (diff < WEEK) {
    const d = Math.floor(diff / DAY)
    return `${d} day${d > 1 ? 's' : ''} ago`
  }
  if (diff < MONTH) {
    const w = Math.floor(diff / WEEK)
    return `${w} week${w > 1 ? 's' : ''} ago`
  }
  const m = Math.floor(diff / MONTH)
  return `${m} month${m > 1 ? 's' : ''} ago`
}
