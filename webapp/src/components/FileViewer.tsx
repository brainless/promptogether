import { createSignal, Show } from "solid-js";
import type { GalleryProjectFile } from "../api/generated/GalleryProjectFile";
import styles from "./FileViewer.module.css";

export default function FileViewer(props: { file: GalleryProjectFile }) {
  const [feedback, setFeedback] = createSignal<string | null>(null);
  let feedbackTimer: ReturnType<typeof setTimeout> | undefined;

  async function copyContent() {
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(props.file.content);
      } else {
        const textarea = document.createElement("textarea");
        textarea.value = props.file.content;
        textarea.style.position = "fixed";
        textarea.style.left = "-9999px";
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand("copy");
        document.body.removeChild(textarea);
      }
      showFeedback("Copied!");
    } catch {
      showFeedback("Failed to copy");
    }
  }

  function showFeedback(message: string) {
    if (feedbackTimer !== undefined) {
      clearTimeout(feedbackTimer);
    }
    setFeedback(message);
    feedbackTimer = setTimeout(() => {
      setFeedback(null);
      feedbackTimer = undefined;
    }, 2000);
  }

  return (
    <div class={styles.fileViewer}>
      <div class={styles.fileHeader}>
        <h3 class={styles.filePath}>{props.file.path}</h3>
        <Show when={props.file.language}>
          <span class={styles.language}>{props.file.language}</span>
        </Show>
      </div>
      <div class={styles.codeWrapper}>
        <button
          class={styles.copyButton}
          onClick={copyContent}
          aria-label={`Copy ${props.file.path} contents`}
          aria-live="polite"
        >
          <Show when={feedback()} fallback="Copy">
            {feedback()}
          </Show>
        </button>
        <pre class={styles.codeBlock}>
          <code>{props.file.content}</code>
        </pre>
      </div>
    </div>
  );
}
