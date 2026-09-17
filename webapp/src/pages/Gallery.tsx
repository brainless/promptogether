import { createMemo, createSignal, For, Show, Loading, Errored } from "solid-js";
import { fetchGalleryProjects } from "../api/client";
import type { GalleryProjectSummary } from "../api/generated/GalleryProjectSummary";
import { paths } from "../router";
import { markNextGalleryProjectFocus } from "../focus-routing";
import styles from "./Gallery.module.css";

export default function Gallery() {
  const [retryCount, setRetryCount] = createSignal(0);
  const projects = createMemo(() => {
    retryCount();
    return fetchGalleryProjects();
  });

  return (
    <div class={styles.gallery}>
      <h1>Gallery</h1>
      <p class={styles.subtitle}>
        Browse projects built with coding agents. See the prompts, the code, and the results.
      </p>

      <Errored
        fallback={(error, reset) => (
          <div class={styles.error} aria-live="polite">
            <p class={styles.errorMessage}>
              Something went wrong loading the gallery. {String(error())}
            </p>
            <button
              class={styles.retryButton}
              onClick={() => {
                setRetryCount((c) => c + 1);
                reset();
              }}
            >
              Retry
            </button>
          </div>
        )}
      >
        <Loading fallback={<p class={styles.loading}>Loading projects…</p>}>
          <Show
            when={projects().length > 0}
            fallback={<p class={styles.empty}>No projects yet. Check back soon.</p>}
          >
            <nav aria-label="Gallery projects">
              <ul class={styles.list}>
                <For each={projects()}>
                  {(project: GalleryProjectSummary) => (
                    <li class={styles.listItem}>
                      <a
                        class={styles.projectLink}
                        href={paths.gallery(project.slug)}
                        aria-label={`${project.title}: ${project.summary}`}
                        onClick={markNextGalleryProjectFocus}
                      >
                        <h2 class={styles.projectTitle}>{project.title}</h2>
                        <p class={styles.projectSummary}>{project.summary}</p>
                      </a>
                    </li>
                  )}
                </For>
              </ul>
            </nav>
          </Show>
        </Loading>
      </Errored>
    </div>
  );
}
