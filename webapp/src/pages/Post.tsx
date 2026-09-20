import { Show } from "solid-js";
import { useParams } from "@solidjs/router";
import { posts } from "virtual:posts";
import { paths } from "../router";
import { formatDate } from "../lib/formatDate";
import YouTubeEmbed from "../components/YouTubeEmbed";
import styles from "./Post.module.css";

export default function Post() {
  const params = useParams<{ slug: string }>();
  const post = () => posts.find((candidate) => candidate.slug === params.slug);

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
          <div class={styles.body} innerHTML={post().html} />
        </article>
      )}
    </Show>
  );
}
