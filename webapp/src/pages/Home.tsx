import styles from "../App.module.css";
import { paths } from "../router";
// import GalleryPreview from "../components/GalleryPreview";

export default function Home() {
  return (
    <>
      <section class={styles.hero} aria-labelledby="welcome-title">
        <p class={styles.eyebrow}>Build what you need</p>
        <h1 id="welcome-title">Turn your ideas<br />into software.<br /><em>I'll show you how.</em></h1>
      </section>

      <section class={styles.aboutMe} aria-labelledby="about-me-title">
        <h2 id="about-me-title" class={styles.srOnly}>About Sumit</h2>
        <img class={styles.aboutMePhoto} src="/SumitDatta_Profile_Picture_Small.jpg" alt="Photo of Sumit" width="150" height="150" />
        <p>Hey, I am Sumit, an engineer for 18 years. I'll help you build software automation that you need, even if you do not have any coding knowledge.</p>
      </section>

      <section class={styles.videoSection} aria-labelledby="video-title">
        <h2 id="video-title" class={styles.srOnly}>Video introduction</h2>
        <div class={styles.videoContainer}>
          <iframe
            src={`https://www.youtube.com/embed/6WvyIpyxEVU`}
            title="promptogether introduction"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
          />
        </div>
      </section>

      {/*<GalleryPreview />*/}

      <section class={styles.section} id="start-a-project" aria-labelledby="start-title" tabindex="-1">
        <h2 id="start-title">Videos on YouTube, reading material here.</h2>
        <p>Every idea starts as a video where I build something real and think out loud along the way. I write up the same ground as posts here on the site, so you can follow along, revisit a step, or skip straight to the part you need.</p>
        <p>Files piling up in your inbox. A team that needs a shared view of who's doing what. A small business that's outgrown its spreadsheets. That's the kind of automation we'll build together.</p>
        <a class={styles.startLink} href={paths.posts()}>Read the posts <span aria-hidden="true">↗</span></a>
      </section>

      <section class={styles.section} aria-labelledby="concepts-title">
        <h2 id="concepts-title">Build a glossary, one video at a time.</h2>
        <p>Coding agents, prompts, context, workflows—the terms pile up fast when you're new to this. Every concept introduced in a video or post gets added to a growing glossary, so you always have somewhere to look things up in plain language.</p>
        <a class={styles.startLink} href={paths.concepts}>Browse concepts <span aria-hidden="true">↗</span></a>
      </section>

      <section class={styles.section} aria-labelledby="use-cases-title">
        <h2 id="use-cases-title">Software for personal, family, and small business use.</h2>
        <p>I'm putting together focused content for people running very small businesses, managing a household, or just trying to keep personal projects organized—practical automation for the everyday, not enterprise software.</p>
        <span class={styles.status}>Use-case series · Coming soon</span>
      </section>

      <section class={styles.section} aria-labelledby="share-title">
        <h2 id="share-title">Tell me what you're trying to build.</h2>
        <p>Soon you'll be able to share your own ideas and needs directly on this site, so we can dig into your specific problem together—not just a generic example.</p>
        <span class={styles.status}>Share your idea · Coming soon</span>
      </section>

      <section class={styles.closing} aria-labelledby="closing-title">
        <p class={styles.eyebrow}>A personal project, made in the open</p>
        <h2 id="closing-title">The tools you need, built by you.</h2>
        <p>promptogether grew out of co-hosted learning sessions I've run for hundreds of people. It exists because everyone deserves the power to create their own software—whether it's for your business, your family, or your daily life.</p>
      </section>
    </>
  );
}
