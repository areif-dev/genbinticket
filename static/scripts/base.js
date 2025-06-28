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
    lastPage.appendChild(e("div", { class: "label", draggable: true }, []));
  }
}

/**
 * Initialize the drag and drop functionality of the labels 
 */
function dragInit() {
  document.querySelectorAll(".label").forEach((el) => {
    el.classList.remove("dragging");
    el.classList.remove("potential-drop");
    // This erases all label event listeners so when we reinit them they don't stack up
    // exponentially and cause performance issues
    el.replaceWith(el.cloneNode(true));
  });

  let source;
  document.querySelectorAll(".label").forEach((label) => {
    label.addEventListener("dragstart", (e) => {
      e.target.classList.add("dragging");
      source = e.currentTarget;
    });

    label.addEventListener("dragenter", (e) => {
      e.preventDefault();
      if (!e.target.classList.contains("dragging")) {
        e.target.classList.add("potential-drop");
      }
    });

    label.addEventListener("dragover", (e) => {
      e.preventDefault();
    });

    label.addEventListener("dragleave", (e) => {
      e.target.classList.remove("potential-drop");
    });

    label.addEventListener("drop", (e) => {
      e.preventDefault();
      const target = e.currentTarget;

      if (source === target) return;

      const sourceClone = source.cloneNode(true);
      const targetClone = target.cloneNode(true);
      source.replaceWith(targetClone);
      target.replaceWith(sourceClone);
    });

    label.addEventListener("dragend", (_) => {
      dragInit();  // We need to reinit here because swapping the nodes erases event listeners
    });
  });
}

/**
 * Append a page of blank labels to the print stack 
 *
 * @param {Event} ev The event that triggered this call
 */
function addPage(ev) {
  const button = ev.currentTarget;
  const body = document.querySelector("body");
  let labels = [];
  for (let i = 0; i < 30; i++) {
    labels.push(e("div", { class: "label", draggable: true }, []));
  }

  const page = e("div", { class: "page" }, labels);
  body.insertBefore(page, button);
  dragInit();
}

/** 
 * Prepare the pages for printing when the user makes a print request
 */
function handlePrint() {
  const pages = document.querySelectorAll(".page");
  pages.forEach((page) => {
    const labels = page.querySelectorAll(".label");
    let hasContent = false;
    for (const label of labels) {
      if (label.hasChildNodes()) {
        hasContent = true;
        break;
      }
    }
    if (!hasContent) {
      page.remove();
    }
  });
}

window.addEventListener("load", () => {
  document.querySelector("#add-page-btn").addEventListener("click", addPage);
  padLabels();
  dragInit();
});

window.addEventListener("beforeprint", handlePrint);
