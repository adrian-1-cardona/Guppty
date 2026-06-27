const content = document.querySelector(".content");
const menuButton = document.querySelector(".mobile-menu");
const sidebar = document.querySelector(".sidebar");
const navLinks = [...document.querySelectorAll(".nav-link")];
const docPagePanels = [...document.querySelectorAll("[data-doc-page-panel]")];
const docPageLinks = [...document.querySelectorAll("[data-doc-page]")];

function setMenu(open) {
  document.body.classList.toggle("menu-open", open);
  menuButton?.setAttribute("aria-expanded", String(open));
}

menuButton?.addEventListener("click", () => {
  setMenu(!document.body.classList.contains("menu-open"));
});

sidebar?.addEventListener("click", (event) => {
  const target = event.target;
  if (target instanceof HTMLAnchorElement && window.matchMedia("(max-width: 820px)").matches) {
    setMenu(false);
  }
});

function activateTabs(buttonSelector, panelSelector, activeValue, buttonAttr, panelAttr) {
  document.querySelectorAll(buttonSelector).forEach((button) => {
    const isActive = button.getAttribute(buttonAttr) === activeValue;
    button.classList.toggle("active", isActive);
    button.setAttribute("aria-selected", String(isActive));
  });

  document.querySelectorAll(panelSelector).forEach((panel) => {
    const isActive = panel.getAttribute(panelAttr) === activeValue;
    panel.classList.toggle("active", isActive);
    panel.hidden = !isActive;
  });
}

document.querySelectorAll(".tab-button").forEach((button) => {
  button.addEventListener("click", () => {
    activateTabs(".tab-button", ".tab-panel", button.dataset.tab, "data-tab", "data-panel");
  });
});

document.querySelectorAll(".syntax-tab").forEach((button) => {
  button.addEventListener("click", () => {
    activateTabs(
      ".syntax-tab",
      ".syntax-panel",
      button.dataset.syntax,
      "data-syntax",
      "data-syntax-panel"
    );
  });
});

async function copyText(text, button) {
  if (!text) return;

  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand("copy");
    textarea.remove();
  }

  const label = button.querySelector("span:last-child");
  const previousText = label?.textContent;
  if (label) label.textContent = "Copied";
  button.classList.add("copied");

  window.setTimeout(() => {
    if (label && previousText) label.textContent = previousText;
    button.classList.remove("copied");
  }, 1200);
}

document.querySelectorAll("[data-copy]").forEach((button) => {
  button.addEventListener("click", () => {
    copyText(button.getAttribute("data-copy"), button);
  });
});

function pageForHash(hash) {
  const matchingLink = navLinks.find((link) => link.getAttribute("href") === hash);
  if (matchingLink) return matchingLink.dataset.docPage;

  const target = document.querySelector(hash);
  const panel = target?.closest("[data-doc-page-panel]");
  return panel?.dataset.docPagePanel || "getting-started";
}

function firstLinkForPage(pageName) {
  return navLinks.find((link) => link.dataset.docPage === pageName);
}

function setActiveNav(hash) {
  navLinks.forEach((link) => {
    link.classList.toggle("active", link.getAttribute("href") === hash);
  });
}

function showDocPage(pageName, hash, scroll = true) {
  const activeHash = hash || firstLinkForPage(pageName)?.getAttribute("href") || "#installation";

  docPagePanels.forEach((panel) => {
    const isActive = panel.dataset.docPagePanel === pageName;
    panel.classList.toggle("active", isActive);
    panel.hidden = !isActive;
  });

  docPageLinks.forEach((link) => {
    if (!(link instanceof HTMLAnchorElement)) return;
    link.setAttribute("aria-current", link.getAttribute("href") === activeHash ? "page" : "false");
  });

  setActiveNav(activeHash);

  if (!scroll || !content) return;

  const target = document.querySelector(activeHash);
  if (target) {
    content.scrollTo({ top: 0, behavior: "auto" });
    window.requestAnimationFrame(() => target.scrollIntoView({ block: "start" }));
  }
}

function navigateToHash(hash, replace = false) {
  const nextHash = hash || "#installation";
  showDocPage(pageForHash(nextHash), nextHash);

  if (replace) {
    history.replaceState(null, "", nextHash);
  } else if (window.location.hash !== nextHash) {
    history.pushState(null, "", nextHash);
  }
}

docPageLinks.forEach((link) => {
  if (!(link instanceof HTMLAnchorElement)) return;

  link.addEventListener("click", (event) => {
    const href = link.getAttribute("href");
    if (!href?.startsWith("#")) return;
    event.preventDefault();
    navigateToHash(href);
  });
});

window.addEventListener("hashchange", () => navigateToHash(window.location.hash, true));

let scrollFrame = 0;

function updateActiveSectionFromScroll() {
  if (!content) return;

  const activePanel = docPagePanels.find((panel) => !panel.hidden);
  const sections = [...(activePanel?.querySelectorAll(".docs-section[id]") || [])];
  const contentTop = content.getBoundingClientRect().top;
  const current =
    sections
      .map((section) => ({
        section,
        distance: Math.abs(section.getBoundingClientRect().top - contentTop - 70),
      }))
      .sort((a, b) => a.distance - b.distance)[0]?.section || sections[0];

  if (current) setActiveNav(`#${current.id}`);
}

content?.addEventListener("scroll", () => {
  window.cancelAnimationFrame(scrollFrame);
  scrollFrame = window.requestAnimationFrame(updateActiveSectionFromScroll);
});

navigateToHash(window.location.hash || "#installation", true);
