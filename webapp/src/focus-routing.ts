let _focusOnNextGalleryProjectMount = false;

export function markNextGalleryProjectFocus() {
  _focusOnNextGalleryProjectMount = true;
}

export function consumeGalleryProjectFocus(): boolean {
  if (_focusOnNextGalleryProjectMount) {
    _focusOnNextGalleryProjectMount = false;
    return true;
  }
  return false;
}
