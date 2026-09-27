declare module "virtual:concepts" {
  export interface ConceptPost {
    slug: string;
    title: string;
    date: string;
  }

  export interface Concept {
    slug: string;
    name: string;
    definition: string;
    url: string | null;
    aliases: string[];
    posts: ConceptPost[];
  }

  export const concepts: Concept[];
}
