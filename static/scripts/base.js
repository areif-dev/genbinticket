/**
 * A shortcut for document.createElement that also sets attributes and can append
 * child elements.
 *
 * @param {string} tagName The type of tag to make this element. This is the
 * argument that is passed to document.createElement
 *
 * @param {object} attrs The attributes to give to this element. E.G.
 * {id: "name-input", class: "form-control"}
 *
 * @param {Node[]} Array of html elements to append as children of
 * this element
 *
 * @returns An HTML Element of type tagName as created by document.createElement
 */
function e(
  tagName,
  attrs,
  children
) {
  let element = document.createElement(tagName);
  for (const attr in attrs) {
    element.setAttribute(attr, attrs[attr]);
  }

  if (children) {
    for (let i = 0; i < children.length; i++) {
      if (typeof children[i] === "string") {
        let textNode = document.createTextNode(children[i] + "");
        element.appendChild(textNode);
      } else {
        element.appendChild(children[i]);
      }
    }
  }

  return element;
}

/**
 * Ensures that there are 30 labels on the last page, even if they are blank. This 
 * will help with drag and drop functionality
 */
function padLabels() {
  const pages = document.querySelectorAll(".page");
  if (pages.length === 0) {
    return;
  }
  const lastPage = pages[pages.length - 1];
  const labels = lastPage.querySelectorAll(".label");
  for (let i = 0; i < (30 - labels.length); i++) {
    lastPage.appendChild(e("div", { class: "label" }, []));
  }
}

window.addEventListener("load", () => {
  padLabels();
});
