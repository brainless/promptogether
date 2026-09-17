import { lazy } from "solid-js";
import { createRouter } from "@solidjs/router";

const Home = lazy(() => import("./pages/Home"));
const GalleryLayout = lazy(() => import("./pages/GalleryLayout"));
const Gallery = lazy(() => import("./pages/Gallery"));
const GalleryProject = lazy(() => import("./pages/GalleryProject"));
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
    { path: "*404", component: NotFound },
  ],
});

export const Router = instance;
export const paths = instance.paths;
