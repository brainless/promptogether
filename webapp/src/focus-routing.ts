import { createContext, createSignal, useContext, type Accessor } from "solid-js";

export type GalleryProjectFocusRequest = {
  id: number;
  slug: string;
};

export type GalleryProjectFocusRouting = {
  request: Accessor<GalleryProjectFocusRequest | null>;
  requestFocus: (slug: string) => void;
  consume: (request: GalleryProjectFocusRequest) => void;
};

export const GalleryProjectFocusContext = createContext<GalleryProjectFocusRouting>();

export function createGalleryProjectFocusRouting(): GalleryProjectFocusRouting {
  const [request, setRequest] = createSignal<GalleryProjectFocusRequest | null>(null);
  let nextRequestId = 0;

  return {
    request,
    requestFocus(slug) {
      setRequest({ id: ++nextRequestId, slug });
    },
    consume(consumedRequest) {
      setRequest((currentRequest) =>
        currentRequest?.id === consumedRequest.id ? null : currentRequest,
      );
    },
  };
}

export function useGalleryProjectFocusRouting(): GalleryProjectFocusRouting {
  return useContext(GalleryProjectFocusContext);
}
