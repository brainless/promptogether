import styles from "./App.module.css";

export default function App() {
  return (
    <div class={styles.page}>
      <a class={styles.skipLink} href="#main">Skip to content</a>
      <header class={styles.header}>
        <a class={styles.brand} href="/" aria-label="Prompt Together home">
          prompt<span class={styles.brandAccent}>ogether</span><span aria-hidden="true">.</span>
        </a>
        <nav aria-label="Main navigation">
          <a class={styles.startLink} href="#start-a-project">Start a Project <span aria-hidden="true">↗</span></a>
        </nav>
      </header>

      <main id="main">
        <section class={styles.hero} aria-labelledby="welcome-title">
          <p class={styles.eyebrow}>A little curiosity goes a long way</p>
          <h1 id="welcome-title">Build something<br />of your own.<br /><em>Learn together.</em></h1>
          <p class={styles.intro}>An open place to learn how to build software with coding agents. Bring an idea, a question, or just yourself.</p>
          <p class={styles.note}>Free to explore. No registration needed.</p>
        </section>

        <section class={styles.section} id="start-a-project" aria-labelledby="start-title" tabindex="-1">
          <p class={styles.eyebrow}>01 / Start small</p>
          <h2 id="start-title">Your idea is a good place to start.</h2>
          <p>A tool for your everyday life. A website for your business. Something you wish existed. Start by putting it into a sentence: “I want to build ___ to help ___.”</p>
          <p>We’re creating guided tours to take you from that first idea to building with a coding agent, one step at a time. No coding experience assumed.</p>
          <span class={styles.status}>Guided tours · Coming soon</span>
        </section>

        <section class={styles.section} aria-labelledby="community-title">
          <p class={styles.eyebrow}>02 / Find your people</p>
          <h2 id="community-title">You don’t have to figure it out alone.</h2>
          <p>Share what you’re trying, see what others are building, and learn from the experiments along the way. There’s room here for first attempts and work in progress.</p>
          <span class={styles.status}>Community forum · Coming soon</span>
        </section>

        <section class={styles.section} aria-labelledby="questions-title">
          <p class={styles.eyebrow}>03 / Ask away</p>
          <h2 id="questions-title">Every question belongs.</h2>
          <p>Unfamiliar words, unexpected errors, or a simple “where do I begin?” A shared Q&A will help you find answers and leave something useful for the next person.</p>
          <span class={styles.status}>Questions & answers · Coming soon</span>
        </section>

        <section class={styles.section} aria-labelledby="help-title">
          <p class={styles.eyebrow}>04 / Keep going</p>
          <h2 id="help-title">A little help when you’re stuck.</h2>
          <p>We’re exploring live AI help to work through issues with you, explain what’s happening, and help you find your next step.</p>
          <span class={styles.status}>Live AI help · In exploration</span>
        </section>

        <section class={styles.closing} aria-labelledby="closing-title">
          <p class={styles.eyebrow}>A space we’re growing together</p>
          <h2 id="closing-title">Curiosity is the only prerequisite.</h2>
          <p>Born from coding agent experiments and learning sessions with hundreds of people. Prompt Together is just getting started. What we learn together will shape what comes next.</p>
        </section>
      </main>

      <footer class={styles.footer}>
        <span>Prompt Together</span>
        <span>For the things you want to make.</span>
      </footer>
    </div>
  );
}
