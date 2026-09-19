import { createMemo, createSignal, For, Show, Loading, Errored } from "solid-js";
import { fetchGalleryProjects } from "../api/client";
import type { GalleryProjectSummary } from "../api/generated";
import { paths } from "../router";
import styles from "../App.module.css";

export default function GalleryPreview() {
  const [activeTab, setActiveTab] = createSignal(0);
  const [retryCount, setRetryCount] = createSignal(0);

  const projects = createMemo(() => {
    retryCount();
    return fetchGalleryProjects();
  });

  const categories = ["All", "Small Business", "Personal Finance", "Productivity"];

  return (
    <section class={styles.galleryPreview} aria-labelledby="gallery-preview-title">
      <p class={styles.eyebrow}>See what others are building</p>
      <h2 id="gallery-preview-title">Gallery</h2>
      <p class={styles.galleryPreviewSubtitle}>
        Browse projects built with coding agents. Get inspired by what's possible.
      </p>

      <div role="tablist" class={styles.tabList} aria-label="Gallery categories">
        <For each={categories}>
          {(category, index) => (
            <button
              role="tab"
              class={`${styles.tab} ${activeTab() === index() ? styles.tabActive : ""}`}
              aria-selected={activeTab() === index() ? "true" : "false"}
              aria-controls={`tabpanel-${index()}`}
              id={`tab-${index()}`}
              onClick={() => setActiveTab(index())}
            >
              {category}
            </button>
          )}
        </For>
      </div>

      <Errored
        fallback={(error, reset) => (
          <div class={styles.galleryError} aria-live="polite">
            <p>Something went wrong loading projects. {String(error())}</p>
            <button
              class={styles.projectLink}
              onClick={() => {
                setRetryCount((c) => c + 1);
                reset();
              }}
            >
              Try again
            </button>
          </div>
        )}
      >
        <Loading fallback={<p class={styles.galleryLoading}>Loading projects…</p>}>
          <Show
            when={projects().length > 0}
            fallback={<p class={styles.galleryLoading}>No projects yet. Check back soon.</p>}
          >
            <For each={categories}>
              {(_category, tabIndex) => (
                <div
                  role="tabpanel"
                  id={`tabpanel-${tabIndex()}`}
                  aria-labelledby={`tab-${tabIndex()}`}
                  class={styles.tabPanel}
                  hidden={activeTab() !== tabIndex()}
                >
                  <For each={projects()}>
                    {(project: GalleryProjectSummary) => (
                      <article class={styles.projectCard}>
                        <h3 class={styles.projectTitle}>{project.title}</h3>
                        <p class={styles.projectSummary}>{project.summary}</p>
                        <a
                          class={styles.projectLink}
                          href={paths.gallery(project.slug)}
                          aria-label={`View ${project.title}`}
                        >
                          View project →
                        </a>
                      </article>
                    )}
                  </For>
                </div>
              )}
            </For>
          </Show>
        </Loading>
      </Errored>

      <a class={styles.galleryLink} href={paths.gallery}>
        Browse all projects →
      </a>
    </section>
  );
}
