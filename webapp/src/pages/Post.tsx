import { For, Show } from "solid-js";
import { useParams } from "@solidjs/router";
import { posts } from "virtual:posts";
import { paths } from "../router";
import { formatDate } from "../lib/formatDate";
import { concepts } from "../concepts";
import YouTubeEmbed from "../components/YouTubeEmbed";
import styles from "./Post.module.css";

export default function Post() {
  const params = useParams<{ slug: string }>();
  const post = () => posts.find((candidate) => candidate.slug === params.slug);
  const related = () => concepts.filter((concept) => concept.posts.some((item) => item.slug === params.slug));

  return (
    <Show
      when={post()}
      fallback={
        <div class={styles.notFound}>
          <h1>Post not found</h1>
          <p>The post you're looking for doesn't exist or may have been removed.</p>
          <a class={styles.backLink} href={paths.posts()}>← All posts</a>
        </div>
      }
    >
      {(post) => (
        <article class={styles.post}>
          <a class={styles.backLink} href={paths.posts()}>← All posts</a>
          <p class={styles.date}>{formatDate(post().date)}</p>
          <h1 class={styles.title}>{post().title}</h1>
          <Show when={post().youtubeUrl}>
            {(url) => <YouTubeEmbed url={url()} title={post().title} />}
          </Show>
          <div class={styles.layout}>
            <div class={styles.body} innerHTML={post().html} />
            <Show when={related().length}>
              <aside class={styles.sidebar} aria-labelledby="post-concepts-title">
                <h2 id="post-concepts-title">Concepts in this post</h2>
                <ul>
                  <For each={related()}>
                    {(concept) => <li><a href={`/concepts#${concept.slug}`}>{concept.name}</a><p>{concept.definition}</p></li>}
                  </For>
                </ul>
              </aside>
            </Show>
          </div>
        </article>
      )}
    </Show>
  );
}
