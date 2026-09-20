import { lazy } from "solid-js";
import { createRouter } from "@solidjs/router";

const Home = lazy(() => import("./pages/Home"));
const GalleryLayout = lazy(() => import("./pages/GalleryLayout"));
const Gallery = lazy(() => import("./pages/Gallery"));
const GalleryProject = lazy(() => import("./pages/GalleryProject"));
const Posts = lazy(() => import("./pages/Posts"));
const Post = lazy(() => import("./pages/Post"));
const NotFound = lazy(() => import("./pages/NotFound"));

const instance = createRouter({
  routes: [
    { path: "/", component: Home },
    {
      path: "/gallery",
      component: GalleryLayout,
      children: [
        { path: "/", component: Gallery },
        { path: "/:slug", component: GalleryProject },
      ],
    },
    {
      path: "/posts",
      children: [
        { path: "/", component: Posts },
        { path: "/:slug", component: Post },
      ],
    },
    { path: "*404", component: NotFound },
  ],
});

export const Router = instance;
export const paths = instance.paths;
