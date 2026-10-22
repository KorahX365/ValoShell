// Stand-in pictures for players without card art. Styles never print initials: the Steam
// styles show Steam's grey silhouette, the VALORANT ones the client's grey V.
import { UserRound } from "lucide-solid";
import styles from "./avatars.module.css";

export function Silhouette() {
  return (
    <span class={styles.silhouette}>
      <UserRound size={20} strokeWidth={1.6} />
    </span>
  );
}
