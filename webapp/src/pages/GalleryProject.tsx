import { createMemo, createSignal, Show, Loading, Errored, onSettled } from "solid-js";
import { useParams } from "@solidjs/router";
import { fetchGalleryProject, NotFoundError } from "../api/client";
import { paths } from "../router";
import { consumeGalleryProjectFocus } from "../focus-routing";
import FileList from "../components/FileList";
import styles from "./GalleryProject.module.css";

export default function GalleryProject() {
  const params = useParams<{ slug: string }>();
  const [retryCount, setRetryCount] = createSignal(0);

  const detail = createMemo(() => {
    retryCount();
    return fetchGalleryProject(params.slug);
  });

  let headingRef!: HTMLHeadingElement;

  onSettled(() => {
    if (consumeGalleryProjectFocus()) {
      headingRef.focus();
    }
  });

  return (
    <div class={styles.detail}>
      <a class={styles.backLink} href={paths.gallery}>
        ← All projects
      </a>

      <Errored
        fallback={(error, reset) => (
          <Show
            when={error() instanceof NotFoundError}
            fallback={
              <div class={styles.error} aria-live="polite">
                <p class={styles.errorMessage}>
                  Something went wrong loading this project. {String(error())}
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
            }
          >
            <div class={styles.notFound} aria-live="polite">
              <h1 class={styles.notFoundHeading}>Project not found</h1>
              <p class={styles.notFoundMessage}>
                The project you're looking for doesn't exist or may have been removed.
              </p>
              <a class={styles.notFoundLink} href={paths.gallery}>
                ← Back to gallery
              </a>
            </div>
          </Show>
        )}
      >
        <Loading fallback={<p class={styles.loading}>Loading project…</p>}>
          <h1 class={styles.title} ref={headingRef} tabindex={-1}>{detail().title}</h1>
          <p class={styles.summary}>{detail().summary}</p>

          <section class={styles.section} aria-labelledby="prompt-heading">
            <h2 class={styles.sectionHeading} id="prompt-heading">
              Prompt
            </h2>
            <pre class={styles.prompt}>{detail().prompt}</pre>
          </section>

          <FileList
            title="Initial Files"
            headingId="initial-files-heading"
            files={detail().initialFiles}
            emptyMessage="No initial files for this project."
          />

          <FileList
            title="Result Files"
            headingId="result-files-heading"
            files={detail().resultFiles}
            emptyMessage="No result files."
          />
        </Loading>
      </Errored>
    </div>
  );
}
