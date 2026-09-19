import styles from "../App.module.css";
import GalleryPreview from "../components/GalleryPreview";

const YOUTUBE_VIDEO_ID = "YOUR_VIDEO_ID_HERE";

export default function Home() {
  return (
    <>
      <section class={styles.hero} aria-labelledby="welcome-title">
        <p class={styles.eyebrow}>Build what you need</p>
        <h1 id="welcome-title">Turn your ideas<br />into software.<br /><em>We'll show you how.</em></h1>
        <p class={styles.intro}>Learn to build the tools, apps, and automations you actually need—no coding background required. Prompt Together helps you go from idea to working software with coding agents.</p>
      </section>

      <section class={styles.videoSection} aria-labelledby="video-title">
        <h2 id="video-title" class={styles.srOnly}>Video introduction</h2>
        <div class={styles.videoContainer}>
          <iframe
            src={`https://www.youtube.com/embed/6WvyIpyxEVU`}
            title="Prompt Together introduction"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
          />
        </div>
      </section>

      <GalleryPreview />

      <section class={styles.section} id="start-a-project" aria-labelledby="start-title" tabindex="-1">
        <p class={styles.eyebrow}>01 / Build what matters to you</p>
        <h2 id="start-title">Start with something useful.</h2>
        <p>A budget tracker for your side business. An invoice generator for your freelance work. A scheduling tool for your team. A personal finance dashboard. Start with the problem you want to solve—we'll help you build it.</p>
        <p>Guided tours walk you through building real software with a coding agent, step by step. No experience needed.</p>
        <span class={styles.status}>Guided tours · Coming soon</span>
      </section>

      <section class={styles.section} aria-labelledby="community-title">
        <p class={styles.eyebrow}>02 / Learn from others building too</p>
        <h2 id="community-title">See what's possible.</h2>
        <p>Browse projects others have built—small business tools, personal productivity apps, finance trackers, and more. Share your own builds, ask questions, and learn from a community of people making software for real life.</p>
        <span class={styles.status}>Community forum · Coming soon</span>
      </section>

      <section class={styles.section} aria-labelledby="questions-title">
        <p class={styles.eyebrow}>03 / Get unstuck fast</p>
        <h2 id="questions-title">Every question gets you closer.</h2>
        <p>Confused by an error? Not sure which tool to use? Ask anything. Our Q&A is built to help you move forward and leave answers that help the next person too.</p>
        <span class={styles.status}>Questions & answers · Coming soon</span>
      </section>

      <section class={styles.section} aria-labelledby="help-title">
        <p class={styles.eyebrow}>04 / Keep building</p>
        <h2 id="help-title">AI help when you're stuck.</h2>
        <p>Get interactive help from AI to debug issues, explain what's happening, and find your next step. We're building this to keep you moving when you hit a wall.</p>
        <span class={styles.status}>Live AI help · In exploration</span>
      </section>

      <section class={styles.closing} aria-labelledby="closing-title">
        <p class={styles.eyebrow}>Empowering builders everywhere</p>
        <h2 id="closing-title">The tools you need, built by you.</h2>
        <p>Built from hands-on learning sessions with hundreds of people. Prompt Together exists because everyone deserves the power to create their own software—whether it's for your business, your finances, or your daily life.</p>
      </section>
    </>
  );
}
