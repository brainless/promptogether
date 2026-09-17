import { For, Show } from "solid-js";
import type { GalleryProjectFile } from "../api/generated";
import FileViewer from "./FileViewer";
import styles from "./FileList.module.css";

function fileFragmentId(headingId: string, path: string, index: number): string {
  const safePath =
    path.replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "") || "file";

  return `${headingId}-file-${index + 1}-${safePath}`;
}

export default function FileList(props: {
  title: string;
  headingId: string;
  files: GalleryProjectFile[];
  emptyMessage: string;
}) {
  return (
    <section class={styles.section} aria-labelledby={props.headingId}>
      <h2 class={styles.sectionHeading} id={props.headingId}>
        {props.title}
      </h2>
      <Show
        when={props.files.length > 0}
        fallback={<p class={styles.emptyMessage}>{props.emptyMessage}</p>}
      >
        <nav class={styles.fileNav} aria-label={`${props.title} files`}>
          <For each={props.files}>
            {(file, index) => (
              <a
                class={styles.fileLink}
                href={`#${fileFragmentId(props.headingId, file.path, index())}`}
              >
                {file.path}
              </a>
            )}
          </For>
        </nav>
        <div class={styles.fileContents}>
          <For each={props.files}>
            {(file, index) => (
              <div id={fileFragmentId(props.headingId, file.path, index())}>
                <FileViewer file={file} />
              </div>
            )}
          </For>
        </div>
      </Show>
    </section>
  );
}
