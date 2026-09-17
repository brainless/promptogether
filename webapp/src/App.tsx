import type { JSX } from "@solidjs/web";
import { paths } from "./router";
import styles from "./App.module.css";

export default function App(props: { children?: JSX.Element }) {
  return (
    <div class={styles.page}>
      <a class={styles.skipLink} href="#main">Skip to content</a>
      <header class={styles.header}>
        <a class={styles.brand} href="/" aria-label="Prompt Together home">
          prompt<span class={styles.brandAccent}>ogether</span><span aria-hidden="true">.</span>
        </a>
        <nav class={styles.nav} aria-label="Main navigation">
          <a class={styles.navLink} href={paths.gallery}>Gallery</a>
          <a class={styles.startLink} href="/#start-a-project">Start a Project <span aria-hidden="true">↗</span></a>
        </nav>
      </header>

      <main id="main">
        {props.children}
      </main>

      <footer class={styles.footer}>
        <span>Prompt Together</span>
        <span>For the things you want to make.</span>
      </footer>
    </div>
  );
}
