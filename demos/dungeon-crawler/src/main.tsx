import ReactDOM from 'react-dom/client';
import App from './App';
import { configureArcheClient } from './api';
import { GAME_CONFIG } from './config';
import './index.css';

configureArcheClient(GAME_CONFIG.arche.apiUrl, GAME_CONFIG.arche.apiKey);

ReactDOM.createRoot(document.getElementById('root')!).render(<App />);
