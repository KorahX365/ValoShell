// Root component: the menu shell routing between scenes (each scene is a Studio stage).
import { Navigate, Route, Router } from "@solidjs/router";
import { MenuShell } from "./menu/MenuShell";
import { StageScene } from "./scenes/StageScene";

export function App() {
  return (
    <Router root={MenuShell}>
      <Route path="/" component={() => <Navigate href="/home" />} />
      <Route path="/home" component={() => <StageScene id="home" />} />
      <Route path="/play" component={() => <StageScene id="play" />} />
      <Route path="/agents" component={() => <StageScene id="agents" />} />
      <Route path="/collection" component={() => <StageScene id="collection" />} />
      <Route path="/career" component={() => <StageScene id="career" />} />
      <Route path="/store" component={() => <StageScene id="store" />} />
      <Route path="/agent-select" component={() => <StageScene id="agent-select" />} />
      <Route path="/live" component={() => <StageScene id="live" />} />
      <Route path="/post-match" component={() => <StageScene id="post-match" />} />
      <Route path="/ingame" component={() => <StageScene id="ingame" />} />
      <Route path="*" component={() => <Navigate href="/home" />} />
    </Router>
  );
}
