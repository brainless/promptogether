import { For, Show } from "solid-js";
import type { GalleryProjectFile } from "../api/generated/GalleryProjectFile";
import FileViewer from "./FileViewer";
import styles from "./FileList.module.css";

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
            {(file) => (
              <a
                class={styles.fileLink}
                href={`#${props.headingId}-${file.path}`}
              >
                {file.path}
              </a>
            )}
          </For>
        </nav>
        <div class={styles.fileContents}>
          <For each={props.files}>
            {(file) => (
              <div id={`${props.headingId}-${file.path}`}>
                <FileViewer file={file} />
              </div>
            )}
          </For>
        </div>
      </Show>
    </section>
  );
}
