import { For, Show } from "solid-js";
import { concepts } from "../concepts";
import { paths } from "../router";
import styles from "./Concepts.module.css";

export default function Concepts() {
  return (
    <div class={styles.page}>
      <h1>Concepts</h1>
      <p class={styles.intro}>A quick guide to terms used in our posts.</p>
      <ul class={styles.list}>
        <For each={concepts}>
          {(concept) => (
            <li id={concept.slug} class={styles.item}>
              <h2>{concept.name}</h2>
              <p>{concept.definition}</p>
              <Show when={concept.url}>
                {(url) => <a class={styles.source} href={url()} target="_blank" rel="noopener noreferrer">Learn more ↗</a>}
              </Show>
              <Show when={concept.posts.length}>
                <h3>In these posts</h3>
                <ul class={styles.postList}>
                  <For each={concept.posts}>
                    {(post) => <li><a href={paths.posts(post.slug)}>{post.title}</a></li>}
                  </For>
                </ul>
              </Show>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
}
