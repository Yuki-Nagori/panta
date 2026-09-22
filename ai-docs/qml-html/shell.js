// HTML 参考件共用壳层；本地模板直接组合，无网络请求或构建步骤。
const shellTemplate = document.createElement("template");
shellTemplate.innerHTML = `
<svg class="icon-symbols" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" focusable="false">
  <defs>
    <symbol id="i-new" viewBox="0 0 15 15" fill="none"><path d="M3 1.5h6l3 3v9H3z" stroke="#4a4a4a" stroke-width="1.2"/><path d="M7.5 6v4M5.5 8h4" stroke="#2e7ce0" stroke-width="1.4"/></symbol>
    <symbol id="i-open" viewBox="0 0 15 15" fill="none"><path d="M1.5 3.5h4l1.5 2h6.5v7h-12z" fill="#f0d491" stroke="#8a6d2f" stroke-width="1.1"/><path d="M1.5 5.5h13" stroke="#8a6d2f" stroke-width="1.1"/></symbol>
    <symbol id="i-save" viewBox="0 0 15 15" fill="none"><path d="M2 2h9l2 2v9H2z" fill="#3d5a80" stroke="#2c3e50" stroke-width="1.1"/><path d="M4.5 2v4h5V2" stroke="#cfd8e3" stroke-width="1.1"/><rect x="4.5" y="8.5" width="6" height="4.5" fill="#cfd8e3"/></symbol>
    <symbol id="i-undo" viewBox="0 0 15 15" fill="none"><path d="M6 3L2.5 6.5 6 10" stroke="#4a4a4a" stroke-width="1.4" stroke-linecap="round"/><path d="M2.5 6.5h5a4 4 0 0 1 4 4v1" stroke="#4a4a4a" stroke-width="1.4" stroke-linecap="round"/></symbol>
    <symbol id="i-redo" viewBox="0 0 15 15" fill="none"><path d="M9 3l3.5 3.5L9 10" stroke="#4a4a4a" stroke-width="1.4" stroke-linecap="round"/><path d="M12.5 6.5h-5a4 4 0 0 0-4 4v1" stroke="#4a4a4a" stroke-width="1.4" stroke-linecap="round"/></symbol>
    <symbol id="i-print" viewBox="0 0 15 15" fill="none"><path d="M4 5V2h7v3M2 5h11v5h-2M4 8h7v5H4z" stroke="#4a4a4a" stroke-width="1.2"/></symbol>
    <symbol id="i-preview" viewBox="0 0 15 15" fill="none"><rect x="1.5" y="2.5" width="12" height="10" stroke="#4a4a4a" stroke-width="1.2"/><path d="M1.5 5.5h12M5.5 5.5v7" stroke="#4a4a4a" stroke-width="1.2"/></symbol>
    <symbol id="i-account" viewBox="0 0 17 17" fill="none"><circle cx="8.5" cy="5.6" r="2.9" stroke="currentColor" stroke-width="1.3"/><path d="M2.8 14.6c.8-3 3-4.5 5.7-4.5s4.9 1.5 5.7 4.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></symbol>
    <symbol id="i-cart" viewBox="0 0 18 18" fill="none"><path d="M2 3h2l1.6 8.4a1 1 0 0 0 1 .8h6.9a1 1 0 0 0 1-.8L15.8 6H4.6" stroke="currentColor" stroke-width="1.3"/><circle cx="7" cy="15" r="1" fill="currentColor"/><circle cx="13" cy="15" r="1" fill="currentColor"/></symbol>
    <symbol id="i-help" viewBox="0 0 17 17" fill="none"><circle cx="8.5" cy="8.5" r="7" stroke="currentColor" stroke-width="1.2"/><path d="M6.6 6.6a2 2 0 1 1 2.7 1.9c-.5.2-.8.6-.8 1.1v.3" stroke="currentColor" stroke-width="1.2"/><circle cx="8.5" cy="12.2" r=".9" fill="currentColor"/></symbol>
    <symbol id="i-minimize" viewBox="0 0 11 11" fill="none"><path d="M1 5.5h9" stroke="currentColor" stroke-width="1.1"/></symbol>
    <symbol id="i-maximize" viewBox="0 0 11 11" fill="none"><rect x="1.5" y="1.5" width="8" height="8" stroke="currentColor" stroke-width="1.1"/></symbol>
    <symbol id="i-close" viewBox="0 0 11 11" fill="none"><path d="M1.5 1.5l8 8M9.5 1.5l-8 8" stroke="currentColor" stroke-width="1.1"/></symbol>
    <symbol id="i-check" viewBox="0 0 15 15" fill="none"><path d="M2.5 8l3 3 7-7.5" stroke="currentColor" stroke-width="1.4"/></symbol>
    <symbol id="i-wizard" viewBox="0 0 15 15" fill="none"><path d="M8 1.5l1.4 3.6L13 6.5l-3.6 1.4L8 11.5 6.6 7.9 3 6.5l3.6-1.4z" stroke="currentColor" stroke-width="1.2"/></symbol>
    <symbol id="i-copy" viewBox="0 0 15 15" fill="none"><rect x="2" y="2" width="8" height="8" stroke="currentColor" stroke-width="1.2"/><rect x="5" y="5" width="8" height="8" stroke="currentColor" stroke-width="1.2"/></symbol>
    <symbol id="i-image" viewBox="0 0 15 15" fill="none"><rect x="1.5" y="2.5" width="12" height="10" stroke="currentColor" stroke-width="1.2"/><circle cx="5" cy="6" r="1.2" fill="currentColor"/><path d="M3 11l3.5-3.5 2.5 2 2-2 3 3.5" stroke="currentColor" stroke-width="1.2"/></symbol>
    <symbol id="i-export" viewBox="0 0 15 15" fill="none"><path d="M7.5 2v8M4.5 7l3 3 3-3" stroke="currentColor" stroke-width="1.3"/><path d="M2 12.5h11" stroke="currentColor" stroke-width="1.3"/></symbol>
    <symbol id="i-delete" viewBox="0 0 15 15" fill="none"><path d="M2.5 4h10M5.5 4V2.5h4V4M4 4l1 9h5l1-9" stroke="currentColor" stroke-width="1.2"/></symbol>
    <symbol id="i-caret" viewBox="0 0 7 5" fill="none"><path d="M0 0h7L3.5 5z" fill="currentColor"/></symbol>
    <symbol id="i-globe" viewBox="0 0 17 17" fill="none"><circle cx="8.5" cy="8.5" r="6.4" stroke="currentColor" stroke-width="1.3"/><ellipse cx="8.5" cy="8.5" rx="2.9" ry="6.4" stroke="currentColor" stroke-width="1.3"/><path d="M2.4 8.5h12.2" stroke="currentColor" stroke-width="1.3"/></symbol>
    <symbol id="i-split" viewBox="0 0 10 12" fill="none"><path d="M1 1.5h8" stroke="currentColor" stroke-width="1.3"/><path d="M2.5 5.5L5 8.8 7.5 5.5z" fill="currentColor"/></symbol>
    <symbol id="i-search" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 14.5 6 5h3l1 9.5M14 14.5 15 5h3l2 9.5M10 10h4"/><rect x="3.5" y="13" width="7" height="7.5" rx="2.5"/><rect x="13.5" y="13" width="7" height="7.5" rx="2.5"/></symbol>
    <symbol id="i-right" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 6 6 6-6 6"/></symbol>
    <symbol id="i-ribbon-project" viewBox="0 0 42 42" fill="none"><path d="M10 8h15l7 7v19H10z" fill="#f2f2f2" stroke="#4a4a4a" stroke-width="1.3"/><path d="M25 8v8h7M21 19v11M15.5 24.5h11" stroke="#2e7ce0" stroke-width="1.6" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-open-project" viewBox="0 0 42 42" fill="none"><path d="M5 13h12l3 4h17v16H5z" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.3"/><path d="M5 17h32" stroke="#4a6f9f" stroke-width="1.3"/><path d="M28 8v13m0 0-4-4m4 4 4-4" stroke="#2e7ce0" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></symbol>
    <symbol id="i-ribbon-new-features" viewBox="0 0 42 42" fill="none"><path d="M21 5a10 10 0 0 1 6 18c-1 .8-1.5 2-1.5 3.3h-9c0-1.3-.5-2.5-1.5-3.3A10 10 0 0 1 21 5z" fill="#f5c518" stroke="#8a6d2f" stroke-width="1.3"/><path d="M16 30h10M17.5 34h7M21 2v2M9 7l2 2M33 7l-2 2" stroke="#f5a623" stroke-width="1.4" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-start-here" viewBox="0 0 42 42" fill="none"><rect x="7" y="9" width="28" height="24" rx="2" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.3"/><path d="M11 14h20M14 20h4M24 20h4M14 26h4M24 26h4" stroke="#2e7ce0" stroke-width="1.4" stroke-linecap="round"/><path d="m19 29 5-4-5-4z" fill="#c8322b"/></symbol>
    <symbol id="i-ribbon-tutorials" viewBox="0 0 42 42" fill="none"><path d="M8 8h11c3 0 4 1 4 4v22c0-3-1-4-4-4H8z" fill="#f2f2f2" stroke="#4a4a4a" stroke-width="1.3"/><path d="M34 8H23c-3 0-4 1-4 4v22c0-3 1-4 4-4h11z" fill="#dce9f7" stroke="#4a4a4a" stroke-width="1.3"/><path d="M12 14h6M26 14h5M26 19h5" stroke="#2e7ce0" stroke-width="1.2" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-videos" viewBox="0 0 42 42" fill="none"><rect x="6" y="10" width="30" height="22" rx="2" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.3"/><path d="m18 15 10 6-10 6z" fill="#2e7ce0"/></symbol>
    <symbol id="i-ribbon-help" viewBox="0 0 42 42" fill="none"><circle cx="21" cy="21" r="13" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.3"/><path d="M17 17a4 4 0 1 1 5 3.8c-1 .4-1.5 1.1-1.5 2.2v1" stroke="#2e7ce0" stroke-width="1.6" stroke-linecap="round"/><circle cx="20.5" cy="28.5" r="1" fill="#2e7ce0"/></symbol>
    <symbol id="i-ribbon-import" viewBox="0 0 26 26" fill="none"><path d="M4 4h8l3 3h7v15H4z" fill="#f0d491" stroke="#8a6d2f" stroke-width="1.2"/><path d="M13 8v9m0 0-3-3m3 3 3-3" stroke="#2e7ce0" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></symbol>
    <symbol id="i-ribbon-add" viewBox="0 0 26 26" fill="none"><rect x="4" y="4" width="18" height="18" rx="2" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.2"/><path d="M13 8v10M8 13h10" stroke="#2e7ce0" stroke-width="1.7" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-dual-domain" viewBox="0 0 26 26" fill="none"><path d="M4 7h8v12H4zM14 7h8v12h-8z" fill="#d9d9d9" stroke="#666" stroke-width="1.1"/><path d="M12 10h2M12 16h2" stroke="#2e7ce0" stroke-width="1.3"/></symbol>
    <symbol id="i-ribbon-geometry" viewBox="0 0 26 26" fill="none"><path d="m5 18 8-12 8 12H5z" stroke="#4a4a4a" stroke-width="1.2"/><circle cx="13" cy="6" r="2" fill="#e6a33a"/><circle cx="5" cy="18" r="2" fill="#5b8bb2"/><circle cx="21" cy="18" r="2" fill="#5b8bb2"/></symbol>
    <symbol id="i-ribbon-mesh" viewBox="0 0 26 26" fill="none"><path d="m4 6 9-3 9 3v14l-9 3-9-3z" stroke="#3f698b" stroke-width="1.1"/><path d="m4 6 9 4 9-4M13 10v13M8.5 4.5v14M17.5 4.5v14" stroke="#6f9fbe" stroke-width="1"/></symbol>
    <symbol id="i-ribbon-thermoplastics-injection-molding" viewBox="0 0 26 26" fill="none"><path d="M13 3 22 8v10l-9 5-9-5V8z" fill="#cbe6da" stroke="#2e4d3a" stroke-width="1.2"/><path d="m8 10 5 3 5-3M13 13v6" stroke="#2e4d3a" stroke-width="1.2"/></symbol>
    <symbol id="i-ribbon-analysis-sequence" viewBox="0 0 26 26" fill="none"><path d="M4 20V6M4 20h18" stroke="#4a4a4a" stroke-width="1.1"/><path d="m6 16 5-5 3 3 6-7" stroke="#2e7ce0" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></symbol>
    <symbol id="i-ribbon-select-material" viewBox="0 0 26 26" fill="none"><path d="M13 4c3.5 4.5 6 7.6 6 11a6 6 0 1 1-12 0c0-3.4 2.5-6.5 6-11z" fill="#f5c518" stroke="#8a6d2f" stroke-width="1.2"/><path d="M10 17c.7 1.2 1.6 1.8 3 2" stroke="#fff" stroke-width="1.1" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-injection-locations" viewBox="0 0 26 26" fill="none"><path d="M13 22s6-6.2 6-11a6 6 0 1 0-12 0c0 4.8 6 11 6 11z" fill="#e8c4c4" stroke="#a33d3d" stroke-width="1.2"/><circle cx="13" cy="11" r="2" fill="#c8322b"/></symbol>
    <symbol id="i-ribbon-process-settings" viewBox="0 0 26 26" fill="none"><path d="M5 7h16M5 13h16M5 19h16" stroke="#4a4a4a" stroke-width="1.2"/><circle cx="10" cy="7" r="2" fill="#2e7ce0"/><circle cx="17" cy="13" r="2" fill="#2e7ce0"/><circle cx="13" cy="19" r="2" fill="#2e7ce0"/></symbol>
    <symbol id="i-ribbon-optimization" viewBox="0 0 26 26" fill="none"><path d="M5 21V5M5 21h16" stroke="#4a4a4a" stroke-width="1.1"/><path d="m7 17 4-4 3 2 6-7" stroke="#2e7ce0" stroke-width="1.5" stroke-linecap="round"/></symbol>
    <symbol id="i-ribbon-boundary-conditions" viewBox="0 0 26 26" fill="none"><rect x="4" y="4" width="18" height="18" stroke="#4a4a4a" stroke-width="1.1" stroke-dasharray="2 2"/><path d="M8 13h10m-3-3 3 3-3 3" stroke="#c8322b" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></symbol>
    <symbol id="i-ribbon-analyze" viewBox="0 0 26 26" fill="none"><circle cx="13" cy="13" r="8" stroke="#777" stroke-width="1.2"/><path d="m11 9 6 4-6 4z" fill="#777"/></symbol>
    <symbol id="i-ribbon-logs" viewBox="0 0 26 26" fill="none"><path d="M6 3h10l4 4v16H6z" fill="#f2f2f2" stroke="#666" stroke-width="1.1"/><path d="M16 3v5h4M9 13h8M9 17h8" stroke="#4a4a4a" stroke-width="1.1"/></symbol>
    <symbol id="i-ribbon-job-manager" viewBox="0 0 26 26" fill="none"><rect x="4" y="5" width="18" height="16" rx="2" fill="#dce9f7" stroke="#4a6f9f" stroke-width="1.1"/><path d="M8 10h10M8 14h10M8 18h6" stroke="#2e7ce0" stroke-width="1.2"/></symbol>
    <symbol id="i-ribbon-results" viewBox="0 0 26 26" fill="none"><path d="M5 21V5M5 21h16" stroke="#4a4a4a" stroke-width="1.1"/><path d="M8 17v-4M13 17V8M18 17v-7" stroke="#2e7ce0" stroke-width="2"/></symbol>
    <symbol id="i-ribbon-reports" viewBox="0 0 26 26" fill="none"><path d="M6 3h10l4 4v16H6z" fill="#f2f2f2" stroke="#666" stroke-width="1.1"/><path d="M16 3v5h4M9 13h8M9 17h6" stroke="#4a4a4a" stroke-width="1.1"/></symbol>
    <symbol id="i-ribbon-shared-views" viewBox="0 0 26 26" fill="none"><path d="m13 3 8 4.5v9L13 21l-8-4.5v-9z" fill="#dce9f7" stroke="#3e6996" stroke-width="1.2"/><path d="m5 7.5 8 4.5 8-4.5M13 12v9" stroke="#3e6996" stroke-width="1.1"/></symbol>
    <symbol id="i-project-file" viewBox="0 0 16 16" fill="none"><path d="M4 1h7l3 3v11H4z" fill="#fff" stroke="#7b858a"/><path d="M11 1v3h3M6 6h6M9 8h3M9 10h3" stroke="#a9b5bc"/><path d="M1 6h4l3 3v4H4l-3-3z" fill="#b9d89b" stroke="#65854c"/><path d="M1 6l3 3h4M4 9v4" stroke="#7d9c61"/></symbol>
    <symbol id="i-project-folder" viewBox="0 0 16 16" fill="none"><path d="M1.5 4h5l1.5 1.7h6.5v8.8h-13z" fill="#e2c47b" stroke="#8a6d2f"/><path d="M1.5 5.7h13" stroke="#8a6d2f"/></symbol>
    <symbol id="i-study" viewBox="0 0 16 16" fill="none"><path d="M3 1.5h7l3 3v11H3z" fill="#fff" stroke="#7b858a"/><path d="M10 1.5v3h3M5 7h6M5 9.5h6M5 12h4" stroke="#6d8ba8"/></symbol>
    <symbol id="i-stl-file" viewBox="0 0 16 16" fill="none"><path d="M3 1.5h7l3 3v11H3z" fill="#e8f0f7" stroke="#4a6f9f"/><path d="M10 1.5v3h3M5 11l2-3 2 2 1.5-2 1.5 3z" stroke="#2e7ce0" stroke-width="1.1" stroke-linejoin="round"/></symbol>
    <symbol id="i-status-ok" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="6.3" fill="#fff" stroke="#5d9b54"/><path d="m4.8 8 2 2 4.4-4.5" stroke="#5d9b54" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></symbol>
    <symbol id="i-task-mesh" viewBox="0 0 16 16" fill="none"><path d="m8 1.5 5 3v7l-5 3-5-3v-7z" stroke="#5c8e66"/><path d="m3 4.5 5 3 5-3M8 7.5v7" stroke="#5c8e66"/></symbol>
    <symbol id="i-task-fill" viewBox="0 0 16 16" fill="none"><path d="M8 1.5c2.2 3.1 4.2 5.7 4.2 8a4.2 4.2 0 1 1-8.4 0c0-2.3 2-4.9 4.2-8z" fill="#f1d36a" stroke="#8a6d2f"/></symbol>
    <symbol id="i-task-material" viewBox="0 0 16 16" fill="none"><circle cx="5" cy="5" r="2.6" fill="#f0d491" stroke="#8a6d2f"/><circle cx="10.5" cy="5" r="2.6" fill="#dce9f7" stroke="#4a6f9f"/><path d="M2.5 13c.6-2 1.4-3 2.5-3s1.9 1 2.5 3M8 13c.6-2 1.4-3 2.5-3s1.9 1 2.5 3" stroke="#5a5a5a"/></symbol>
    <symbol id="i-task-injection" viewBox="0 0 16 16" fill="none"><path d="M8 1.5 13 6.5 8 14.5 3 6.5z" fill="#dce9f7" stroke="#4a6f9f"/><circle cx="8" cy="6.5" r="1.4" fill="#c8322b"/></symbol>
    <symbol id="i-task-settings" viewBox="0 0 16 16" fill="none"><path d="M2 4h12M2 8h12M2 12h12" stroke="#4a4a4a"/><circle cx="5" cy="4" r="1.5" fill="#2e7ce0"/><circle cx="11" cy="8" r="1.5" fill="#2e7ce0"/><circle cx="7" cy="12" r="1.5" fill="#2e7ce0"/></symbol>
    <symbol id="i-task-optimization" viewBox="0 0 16 16" fill="none"><path d="M3 13V3M3 13h10" stroke="#4a4a4a"/><path d="m5 10 2-2 2 1 3-4" stroke="#2e7ce0" stroke-width="1.3" stroke-linecap="round"/></symbol>
    <symbol id="i-task-analysis" viewBox="0 0 16 16" fill="none"><path d="M3 13V3M3 13h10" stroke="#5c8e66"/><path d="m5 10 2-3 2 2 3-5" stroke="#5c8e66" stroke-width="1.3" stroke-linecap="round"/></symbol>
    <symbol id="i-log" viewBox="0 0 16 16" fill="none"><path d="M3 1.5h7l3 3v11H3z" fill="#fff" stroke="#7b858a"/><path d="M10 1.5v3h3M5 8h6M5 10.5h5" stroke="#7b858a"/></symbol>
  </defs>
</svg>

<header class="top-chrome">
  <div class="brand" aria-label="panta">P</div>
  <div class="titlebar">
    <div class="titlebar-content">
      <div class="tool-group" role="group" aria-label="Quick actions">
        <button type="button" class="tool-button" aria-label="New document" title="New document"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-new"/></svg><svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
        <button type="button" class="tool-button" aria-label="Open document" title="Open document"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-open"/></svg></button>
        <button type="button" class="tool-button" aria-label="Save document" title="Save document"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-save"/></svg><svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
        <button type="button" class="tool-button" aria-label="Undo" title="Undo"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-undo"/></svg></button>
        <button type="button" class="tool-button" aria-label="Redo" title="Redo"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-redo"/></svg></button>
        <button type="button" class="tool-button" aria-label="Print" title="Print"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-print"/></svg></button>
        <button type="button" class="tool-button" aria-label="Preview animation" title="Preview animation"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-preview"/></svg><svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
        <button type="button" class="tool-button activate">Activate Animation (A)<svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
        <svg class="icon" aria-hidden="true" focusable="false"><use href="#i-split"/></svg>
      </div>
      <span class="caption"></span>
      <div class="tool-group" role="group" aria-label="Search and account">
        <label class="search-field">
          <svg class="icon" aria-hidden="true" focusable="false"><use href="#i-right"/></svg>
          <input type="search" aria-label="Search" placeholder="Enter a keyword or phrase">
        </label>
        <button type="button" class="tool-button" aria-label="Search" title="Search"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-search"/></svg></button>
        <button type="button" class="tool-button"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-account"/></svg>Sign in<svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
        <button type="button" class="tool-button" aria-label="Shopping cart" title="Shopping cart"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-cart"/></svg></button>
        <button type="button" class="tool-button" aria-label="Help" title="Help"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-help"/></svg><svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
      </div>
      <div class="window-controls" role="group" aria-label="Window controls">
        <button type="button" class="tool-button" aria-label="Minimize" title="Minimize"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-minimize"/></svg></button>
        <button type="button" class="tool-button" aria-label="Maximize" title="Maximize"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-maximize"/></svg></button>
        <button type="button" class="tool-button window-close" aria-label="Close window" title="Close window"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-close"/></svg></button>
      </div>
    </div>
  </div>
  <nav class="menubar" aria-label="Main menu">
    <template data-slot="menu"></template>
    <button type="button" class="language" aria-label="Language" title="Language"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-globe"/></svg><svg class="icon caret" aria-hidden="true" focusable="false"><use href="#i-caret"/></svg></button>
  </nav>
</header>

<template data-slot="ribbon"></template>

<main class="workspace">
  <aside class="left-column" aria-label="Project panels">
    <section class="panel tasks-panel" aria-label="Tasks">
      <button type="button" class="pane-close" aria-label="Close tasks" title="Close tasks"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-close"/></svg></button>
      <div class="tabs tabs-closable">
        <div class="tab-strip" role="tablist" aria-label="Task panel">
          <button type="button" role="tab" aria-selected="true" tabindex="0">Tasks</button>
          <button type="button" role="tab" aria-selected="false" tabindex="-1">Tools</button>
          <button type="button" role="tab" aria-selected="false" tabindex="-1">Shared Views</button>
        </div>
      </div>
      <div class="panel-content">
        <template data-slot="tasks"></template>
      </div>
    </section>
    <section class="panel output-panel" aria-label="Output">
      <button type="button" class="pane-close" aria-label="Close output" title="Close output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-close"/></svg></button>
      <div class="output-toolbar" role="group" aria-label="Output actions">
        <button type="button" class="tool-button" aria-label="New output" title="New output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-new"/></svg></button>
        <button type="button" class="tool-button" aria-label="Open output" title="Open output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-open"/></svg></button>
        <button type="button" class="tool-button" aria-label="Save output" title="Save output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-save"/></svg></button>
        <button type="button" class="tool-button" aria-label="Check output" title="Check output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-check"/></svg></button>
        <button type="button" class="tool-button" aria-label="Output wizard" title="Output wizard"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-wizard"/></svg></button>
        <button type="button" class="tool-button" aria-label="Clear output" title="Clear output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-close"/></svg></button>
        <button type="button" class="tool-button" aria-label="Copy output" title="Copy output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-copy"/></svg></button>
        <button type="button" class="tool-button" aria-label="Image" title="Image"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-image"/></svg></button>
        <button type="button" class="tool-button" aria-label="Export output" title="Export output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-export"/></svg></button>
        <button type="button" class="tool-button" aria-label="Delete output" title="Delete output"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-delete"/></svg></button>
      </div>
      <div class="panel-content"></div>
    </section>
  </aside>
  <section class="panel viewport" aria-label="Viewport">
    <button type="button" class="pane-close" aria-label="Close viewport" title="Close viewport"><svg class="icon" aria-hidden="true" focusable="false"><use href="#i-close"/></svg></button>
    <div class="panel-content"><template data-slot="viewport"></template></div>
    <div class="tabs tabs-bottom">
      <div class="tab-strip" role="tablist" aria-label="Viewport">
        <button type="button" role="tab" aria-selected="true" tabindex="0">Model</button>
        <button type="button" role="tab" aria-selected="false" tabindex="-1">Mesh</button>
        <button type="button" role="tab" aria-selected="false" tabindex="-1">Results</button>
      </div>
    </div>
  </section>
</main>

<template data-slot="dialog"></template>
<footer class="statusbar">Ready</footer>
`;

const shell = shellTemplate.content.cloneNode(true);
shell.querySelector(".caption").textContent = document.title;
shell.querySelectorAll("[data-slot]").forEach((slot) => {
  const content = document.getElementById("page-" + slot.dataset.slot);
  if (content) slot.replaceWith(content.content.cloneNode(true));
  else slot.remove();
});
document.body.append(shell);

const closeDialog = (dialog) => {
  if (dialog) dialog.hidden = true;
};

document.querySelectorAll("[data-dialog]").forEach((trigger) => {
  trigger.addEventListener("click", () => {
    const dialog = document.querySelector(`[data-dialog-panel="${trigger.dataset.dialog}"]`);
    if (!dialog) return;
    dialog.hidden = false;
    dialog.querySelector("input, select, button")?.focus();
  });
});

document.querySelectorAll("[data-dialog-close]").forEach((trigger) => {
  trigger.addEventListener("click", () => closeDialog(trigger.closest("[data-dialog-panel]")));
});

document.querySelectorAll("[data-file-picker]").forEach((trigger) => {
  trigger.addEventListener("click", () => {
    const picker = document.querySelector(`[data-import-file-input="${trigger.dataset.filePicker}"]`);
    picker?.click();
  });
});

document.querySelectorAll("[data-import-file-input]").forEach((picker) => {
  picker.addEventListener("change", () => {
    const file = picker.files?.[0];
    if (!file) return;
    document.querySelectorAll("[data-import-file-name]").forEach((target) => {
      target.textContent = file.name;
    });
    const dialog = document.querySelector('[data-dialog-panel="import"]');
    if (dialog) {
      dialog.hidden = false;
      dialog.querySelector("select")?.focus();
    }
  });
});

document.querySelectorAll("[data-demo-navigate]").forEach((trigger) => {
  trigger.addEventListener("click", () => {
    window.location.href = trigger.dataset.demoNavigate;
  });
});

// 仅演示页签选中状态；工程命令和视图内容由后续 QML 实现承接。
document.querySelectorAll('[role="tablist"]').forEach((bar) => {
  const tabs = [...bar.querySelectorAll('[role="tab"]')];
  const selectTab = (index) => {
    bar.style.setProperty("--tab-index", index);
    tabs.forEach((tab, position) => {
      const selected = position === index;
      tab.setAttribute("aria-selected", String(selected));
      tab.tabIndex = selected ? 0 : -1;
    });
  };
  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => selectTab(index));
    tab.addEventListener("keydown", (event) => {
      let next = index;
      if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
      else if (event.key === "ArrowLeft") next = (index + tabs.length - 1) % tabs.length;
      else if (event.key === "Home") next = 0;
      else if (event.key === "End") next = tabs.length - 1;
      else return;
      event.preventDefault();
      selectTab(next);
      tabs[next].focus();
    });
  });
});
