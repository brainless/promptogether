import { For, Show } from "solid-js";
import { posts } from "virtual:posts";
import { paths } from "../router";
import { formatDate } from "../lib/formatDate";
import styles from "./Posts.module.css";

export default function Posts() {
  return (
    <div class={styles.postsIndex}>
      <h1>Posts</h1>
      <p class={styles.subtitle}>Notes, updates, and videos from building Prompt Together.</p>

      <Show
        when={posts.length > 0}
        fallback={<p class={styles.empty}>No posts yet. Check back soon.</p>}
      >
        <ul class={styles.list}>
          <For each={posts}>
            {(post) => (
              <li class={styles.listItem}>
                <a class={styles.postLink} href={paths.posts(post.slug)}>
                  <span class={styles.postDate}>{formatDate(post.date)}</span>
                  <h2 class={styles.postTitle}>{post.title}</h2>
                  <p class={styles.postDescription}>{post.description}</p>
                </a>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </div>
  );
}
