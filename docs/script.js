const menuButton = document.querySelector(".mobile-menu");
const sidebar = document.querySelector(".sidebar");
const navLinks = document.querySelectorAll(".nav-link");

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

const sections = [...document.querySelectorAll(".docs-section[id]")];

if ("IntersectionObserver" in window && sections.length) {
  const observer = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((entry) => entry.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];

      if (!visible) return;

      navLinks.forEach((link) => {
        link.classList.toggle("active", link.getAttribute("href") === `#${visible.target.id}`);
      });
    },
    {
      rootMargin: "-18% 0px -70% 0px",
      threshold: [0.05, 0.2, 0.6],
    }
  );

  sections.forEach((section) => observer.observe(section));
}
