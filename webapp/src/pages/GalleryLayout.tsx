import type { JSX } from "@solidjs/web";
import { createEffect, createMemo, createSignal, For, Show, Loading, Errored } from "solid-js";
import { useLocation } from "@solidjs/router";
import { fetchGalleryProjects } from "../api/client";
import type { GalleryProjectSummary } from "../api/generated";
import { paths } from "../router";
import {
  createGalleryProjectFocusRouting,
  GalleryProjectFocusContext,
} from "../focus-routing";
import styles from "./GalleryLayout.module.css";

export default function GalleryLayout(props: { children?: JSX.Element }) {
  const location = useLocation<{ slug?: string }>();
  const [retryCount, setRetryCount] = createSignal(0);
  const focusRouting = createGalleryProjectFocusRouting();
  const projects = createMemo(() => {
    retryCount();
    return fetchGalleryProjects();
  });

  const activeSlug = createMemo(() => {
    const match = location.pathname.match(/^\/gallery\/([^/]+)$/);
    return match ? match[1] : null;
  });

  createEffect(
    () => activeSlug(),
    (slug, previousSlug) => {
      const request = focusRouting.request();
      if (request && previousSlug === request.slug && slug !== request.slug) {
        focusRouting.consume(request);
      }
    },
  );

  return (
    <GalleryProjectFocusContext value={focusRouting}>
      <div class={styles.galleryLayout}>
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
            <div class={styles.columns} data-has-detail={activeSlug() ? "" : undefined}>
              <nav class={styles.sidebar} aria-label="Gallery projects">
                <Show
                  when={projects().length > 0}
                  fallback={<p class={styles.empty}>No projects yet. Check back soon.</p>}
                >
                  <ul class={styles.list}>
                    <For each={projects()}>
                      {(project: GalleryProjectSummary) => (
                        <li class={styles.listItem}>
                          <a
                            class={styles.projectLink}
                            href={paths.gallery(project.slug)}
                            aria-label={`${project.title}: ${project.summary}`}
                            aria-current={activeSlug() === project.slug ? "page" : undefined}
                            onClick={(event) => {
                              if (
                                event.button === 0 &&
                                !event.metaKey &&
                                !event.ctrlKey &&
                                !event.shiftKey &&
                                !event.altKey
                              ) {
                                focusRouting.requestFocus(project.slug);
                              }
                            }}
                          >
                            <h2 class={styles.projectTitle}>{project.title}</h2>
                            <p class={styles.projectSummary}>{project.summary}</p>
                          </a>
                        </li>
                      )}
                    </For>
                  </ul>
                </Show>
              </nav>

              <div class={styles.content}>
                {props.children}
              </div>
            </div>
          </Loading>
        </Errored>
      </div>
    </GalleryProjectFocusContext>
  );
}
