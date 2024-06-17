import "./App.css";
import { Converters } from "./Converters";
import { Header } from "./Header";
import { Pitch1, Pitch2 } from "./Pitches";

function App() {
  return (
    <div style={{ display: "grid" }}>
      <Header />
      <Pitch1 />
      <Converters />
      <Pitch2 />
    </div>
  );
}

export default App;
