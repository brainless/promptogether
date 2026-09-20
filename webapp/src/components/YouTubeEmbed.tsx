import { Show, createMemo } from "solid-js";
import { extractYouTubeId } from "../lib/youtube";
import styles from "./YouTubeEmbed.module.css";

export default function YouTubeEmbed(props: { url: string; title: string }) {
  const videoId = createMemo(() => extractYouTubeId(props.url));

  return (
    <Show when={videoId()}>
      {(id) => (
        <div class={styles.wrapper}>
          <iframe
            class={styles.frame}
            src={`https://www.youtube-nocookie.com/embed/${id()}`}
            title={props.title}
            loading="lazy"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            allowfullscreen
          />
        </div>
      )}
    </Show>
  );
}
