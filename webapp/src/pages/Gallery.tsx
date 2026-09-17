import styles from "./Gallery.module.css";

export default function Gallery() {
  return (
    <div class={styles.galleryIndex}>
      <p class={styles.galleryHint}>
        Select a project from the list to view its details.
      </p>
    </div>
  );
}
