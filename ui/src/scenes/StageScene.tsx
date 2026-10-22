// A scene is a Studio stage: its pieces are the scene. One component serves every route.
import { Stage } from "~/studio/Stage";
import type { StageId } from "~/studio/types";

export function StageScene(props: { id: StageId }) {
  return <Stage id={props.id} />;
}
