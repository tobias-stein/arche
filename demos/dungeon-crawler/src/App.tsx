import { GameComponent } from './game/GameComponent';
import ActivityLog from './components/ActivityLog';
import MiniMap from './components/MiniMap';

function App() {
  return (
    <>
      <GameComponent />
      <ActivityLog />
      <MiniMap />
    </>
  );
}

export default App;
