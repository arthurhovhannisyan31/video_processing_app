export const stopImmediatePropagation = (e: Event): void => {
  e.preventDefault();
  e.stopPropagation();
};
