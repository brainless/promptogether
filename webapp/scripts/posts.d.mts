export interface Post {
  slug: string;
  title: string;
  date: string;
  description: string;
  youtubeUrl: string | null;
  html: string;
}

export function loadPosts(contentDir: string): Post[];
