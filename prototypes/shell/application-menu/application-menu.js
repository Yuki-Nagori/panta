// 原型演示平台菜单、统一 Settings 弹窗和键盘交互，不连接真实服务。
const platformButtons = [...document.querySelectorAll("[data-platform-choice]")];
const brand = document.querySelector(".brand");
const desktopTrigger = document.createElement("button");
const macMenuBar = document.querySelector(".mac-system-menubar");
const macAppTrigger = document.querySelector(".mac-app-menu-trigger");
const macFileTrigger = document.querySelector("[data-menubar='file']");
const macMenus = [
  document.querySelector(".mac-application-menu"),
  document.querySelector(".mac-file-menu")
];
const backstage = document.querySelector(".backstage");
const backstageTitle = document.querySelector("#backstage-title");
const backstageCommands = [...document.querySelectorAll(".backstage-command")];
const backstageViews = [...document.querySelectorAll("[data-backstage-view]")];
const modalBackground = [".prototype-controls", ".top-chrome", ".mac-system-menubar", ".ribbon", ".workspace", ".statusbar"]
  .map(selector => document.querySelector(selector))
  .filter(Boolean);
let activeMacMenu = null;
let activeMacTrigger = null;

desktopTrigger.type = "button";
desktopTrigger.className = "brand-menu-trigger";
desktopTrigger.setAttribute("aria-label", "Open application menu");
desktopTrigger.setAttribute("aria-haspopup", "dialog");
desktopTrigger.setAttribute("aria-expanded", "false");
desktopTrigger.textContent = "P";
brand.replaceChildren(desktopTrigger);

function currentPlatformTrigger() {
  return document.body.dataset.platform === "macos" ? macAppTrigger : desktopTrigger;
}

function setBackgroundInert(inert, includeBackstage = false) {
  modalBackground.forEach(element => { element.inert = inert; });
  if (includeBackstage) backstage.inert = inert;
}

function closeMacMenus(restoreFocus = false) {
  macMenus.forEach(menu => { menu.hidden = true; });
  [macAppTrigger, macFileTrigger].forEach(trigger => trigger.setAttribute("aria-expanded", "false"));
  activeMacMenu = null;
  const trigger = activeMacTrigger;
  activeMacTrigger = null;
  if (restoreFocus) (trigger || currentPlatformTrigger()).focus();
}

function openMacMenu(menu, trigger) {
  closeBackstage();
  closeMacMenus();
  activeMacMenu = menu;
  activeMacTrigger = trigger;
  menu.hidden = false;
  trigger.setAttribute("aria-expanded", "true");
  menu.querySelector("[role='menuitem']:not(:disabled)").focus();
}

function closeBackstage(restoreFocus = false) {
  backstage.hidden = true;
  backstage.inert = false;
  setBackgroundInert(false);
  desktopTrigger.setAttribute("aria-expanded", "false");
  if (restoreFocus) currentPlatformTrigger().focus();
}

function openBackstage() {
  closeMacMenus();
  if (brand.getClientRects().length > 0) {
    backstage.style.top = `${brand.getBoundingClientRect().top}px`;
  } else {
    backstage.style.removeProperty("top");
  }
  backstage.hidden = false;
  backstage.inert = false;
  desktopTrigger.setAttribute("aria-expanded", "true");
  setBackgroundInert(true);
  setBackstagePage("recent");
  backstage.querySelector(".backstage-command:not(:disabled)")?.focus();
}

function setBackstagePage(page) {
  const titles = { recent: "Recent Projects", new: "New Project", open: "Open Project" };
  backstageTitle.textContent = titles[page] || titles.recent;
  backstageViews.forEach(view => { view.hidden = view.dataset.backstageView !== page; });
  backstageCommands.forEach(command => {
    command.classList.toggle("is-current", command.dataset.backstagePage === page);
  });
}

function setPlatform(platform) {
  closeBackstage();
  closeMacMenus();
  document.body.dataset.platform = platform;
  backstage.style.removeProperty("top");
  macMenuBar.hidden = platform !== "macos";
  platformButtons.forEach(button => {
    button.setAttribute("aria-pressed", String(button.dataset.platformChoice === platform));
  });
}

platformButtons.forEach(button => button.addEventListener("click", () => setPlatform(button.dataset.platformChoice)));
desktopTrigger.addEventListener("click", () => backstage.hidden ? openBackstage() : closeBackstage());
document.querySelector("[data-close-backstage]").addEventListener("click", () => closeBackstage(true));
macAppTrigger.addEventListener("click", () => {
  macAppTrigger.getAttribute("aria-expanded") === "true"
    ? closeMacMenus()
    : openMacMenu(macMenus[0], macAppTrigger);
});
macFileTrigger.addEventListener("click", () => {
  macFileTrigger.getAttribute("aria-expanded") === "true"
    ? closeMacMenus()
    : openMacMenu(macMenus[1], macFileTrigger);
});

backstageCommands.forEach(command => command.addEventListener("click", () => {
  if (command.dataset.backstagePage || command.dataset.command) {
    setBackstagePage(command.dataset.backstagePage || command.dataset.command);
  }
}));

document.querySelector(".backstage-exit").addEventListener("click", () => closeBackstage());

document.querySelectorAll("[data-dialog-frame='settings']").forEach(button => {
  button.addEventListener("click", () => {
    closeBackstage();
    closeMacMenus();
    setBackgroundInert(true, true);
  }, true);
});

document.addEventListener("panta:dialog-frame-closed", event => {
  if (event.detail?.id === "settings") {
    setBackgroundInert(false, true);
    currentPlatformTrigger().focus();
  }
});

document.querySelectorAll(".mac-file-menu [data-command]").forEach(item => item.addEventListener("click", () => {
  const page = item.dataset.command;
  closeMacMenus();
  openBackstage();
  setBackstagePage(page);
}));

document.addEventListener("pointerdown", event => {
  if (!backstage.hidden && !backstage.contains(event.target) && !desktopTrigger.contains(event.target)) {
    closeBackstage();
  }
  if (activeMacMenu && !activeMacMenu.contains(event.target)
      && !macAppTrigger.contains(event.target) && !macFileTrigger.contains(event.target)) {
    closeMacMenus();
  }
});

document.addEventListener("keydown", event => {
  if (event.key === "Escape") {
    if (!backstage.hidden) closeBackstage(true);
    else if (activeMacMenu) closeMacMenus(true);
    return;
  }

  if (event.key === "ArrowDown" && document.activeElement === desktopTrigger) {
    openBackstage();
    return;
  }
  if (event.key === "ArrowDown" && document.activeElement === macAppTrigger) {
    openMacMenu(macMenus[0], macAppTrigger);
    return;
  }
  if (event.key === "ArrowDown" && document.activeElement === macFileTrigger) {
    openMacMenu(macMenus[1], macFileTrigger);
    return;
  }

  if (activeMacMenu && ["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
    const items = [...activeMacMenu.querySelectorAll("[role='menuitem']:not(:disabled)")];
    const currentIndex = items.indexOf(document.activeElement);
    let nextIndex = currentIndex;
    if (event.key === "ArrowDown") nextIndex = (currentIndex + 1) % items.length;
    if (event.key === "ArrowUp") nextIndex = (currentIndex - 1 + items.length) % items.length;
    if (event.key === "Home") nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;
    event.preventDefault();
    items[nextIndex]?.focus();
    return;
  }

  if (event.key !== "Tab") return;
  if (activeMacMenu) {
    closeMacMenus(true);
    return;
  }
  const surface = !backstage.hidden ? backstage : null;
  if (!surface) return;
  const focusable = [...surface.querySelectorAll("button:not(:disabled), input, select, [tabindex='0']")]
    .filter(element => !element.hidden && element.getClientRects().length > 0);
  if (focusable.length === 0) return;
  const first = focusable[0];
  const last = focusable.at(-1);
  if (event.shiftKey && (document.activeElement === first || !surface.contains(document.activeElement))) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && (document.activeElement === last || !surface.contains(document.activeElement))) {
    event.preventDefault();
    first.focus();
  }
});

setPlatform(document.body.dataset.platform || "desktop");
