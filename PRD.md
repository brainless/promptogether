# Prompt Together — High-Level Product Requirements

**Project:** `promptogether` (Prompt Together)  
**Website:** `https://promptogether.com`
**Repository:** `git@github.com:brainless/promptogether.git`
**Status:** Initial direction; features and scope will evolve through experimentation and community feedback.

## Vision

An inclusive place where people learn to build software with coding agents, for personal projects or business needs. Prompt Together should make getting started approachable and help people keep moving when they get stuck.

The platform grows out of hands-on experimentation with coding agents and co-hosted learning sessions attended by hundreds of people.

## Audience and Need

People curious about building software with AI often need help choosing a starting point, understanding unfamiliar concepts, and resolving problems along the way. The platform should welcome beginners, people without a technical background, and experienced builders exploring new ways to work.

## Product Principles

- **Open access:** Free to explore, with no registration required to get started or access learning content.
- **Inclusive learning:** Use approachable language, accessible interfaces, and guidance that does not assume prior coding experience.
- **Learn by building:** Help people make useful software and understand what their coding agents are doing.
- **Learn together:** Make room for questions, shared experience, and mutual support.
- **Evolve through use:** Treat the initial feature ideas as a direction to explore, rather than a fixed roadmap.

## Initial Product Areas

| Area | Intended outcome |
| --- | --- |
| Guided tours | Help people get started with coding agents and work toward a first useful software project. |
| Community forum | Give people a place to discuss experiments, share projects, and learn from each other. |
| Questions and answers | Help people find answers to specific problems and contribute solutions others can reuse. |
| Live AI help | Offer interactive assistance when people encounter issues while learning or building. |

The boundaries and priority of these areas will be refined as the product develops. The platform should also support continued learning between the existing co-hosted sessions; hosting live events on the website is not yet a defined requirement.

## Core Experience

A visitor can arrive without an account, understand what the platform offers, and begin a guided tour. As they build, they can explore community discussions and existing answers, or seek help through the support experiences available. Over time, learners can share what they have learned and help others progress.

## Technical Direction

- **Backend:** Rust.
- **Web application:** SolidJS 2.0 with component CSS.
- **Development approach:** Build and iterate with coding agents.

Detailed architecture, infrastructure, data models, and provider choices will be decided during implementation.

## Early Signs of Success

- Visitors can begin learning without registration or facilitator assistance.
- Learners complete guided experiences and make progress on useful projects.
- People can find or receive help that resolves their issues.
- Community participation is welcoming and produces knowledge others can reuse.

Specific metrics and targets will be defined after the first experiences are tested with learners.

## Decisions to Revisit

- The first guided tour, supported coding agents, and initial release scope.
- Whether contributing posts or questions needs an identity, while keeping exploration registration-free.
- How forum discussions and Q&A fit together.
- Community moderation and expectations for participation.
- The scope, reliability, privacy, and operating cost of live AI assistance.

This document establishes product intent. Detailed feature requirements and delivery milestones will follow as the project takes shape.
