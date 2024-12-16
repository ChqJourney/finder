<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow,Window } from "@tauri-apps/api/window";
    import type { SearchResult, SearchScenarios } from "../biz/types";
    import { message } from '@tauri-apps/plugin-dialog';
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { load, Store } from "@tauri-apps/plugin-store";
    import { open } from "@tauri-apps/plugin-shell";
    import { register, unregister, unregisterAll,isRegistered } from "@tauri-apps/plugin-global-shortcut";
    import { preventDefault } from "svelte/legacy";

let isAlwaysOnTop=$state<boolean>(false);
let searchScenarios=$state<SearchScenarios[]>([]);
let currentSearchState=$state<SearchScenarios|null>(null);
let isDropdownOpen=$state<boolean>(false);
let searchResults=$state<SearchResult[]>([]);
let isSearching=$state<boolean>(false);
let searchTerm=$state<string>('');
let selectedIndex=$state<number>(0);
let store:Store;
let globalShortcutData:any;
let usedPath=$state<string[]|undefined>([]);
let resultsContainer: HTMLDivElement;
const selectSearchState=async(state:SearchScenarios)=>{
  currentSearchState=state;
  isDropdownOpen=false;
await store.set('lastSearchState', state.name);
// store.save();
}

async function search() {
  if (!currentSearchState || !searchTerm.trim()) return;
  
  isSearching = true;
  try {
    console.log("search")
    searchResults = [];
    const results = await invoke<SearchResult[]>("search", {
      searchTerm,
      path: currentSearchState.path,
      level: currentSearchState.level,
      target: currentSearchState.target,
      fileExtensions: currentSearchState.fileExtensions
    });
    console.log(results)
    
    // Split results into used and unused paths
    const usedResults = results.filter(result => usedPath?.includes(result.path));
    const unusedResults = results.filter(result => !usedPath?.includes(result.path));
    
    // Sort used results by path
    const sortedUsedResults = usedResults.sort((a, b) => a.path.localeCompare(b.path));
    
    // Combine sorted used results with unused results
    searchResults = [...sortedUsedResults, ...unusedResults];
  } catch (error) {
    console.error("Search error:", error);
    searchResults = [];
  } finally {
    isSearching = false;
  }
}

async function cancelSearch() {
  try {
    await invoke("cancel_search");
  } catch (error) {
    console.error("Cancel search error:", error);
  }
}
function clearSearch() {
  searchTerm = '';
  searchResults = [];
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    console.log('Enter key pressed');
    search();
  }
}

async function openFile(path: string) {
  try {
    await open(path);
    usedPath=[...new Set([...(usedPath??[]),path])];
    await store.set('usedPath', usedPath);
  } catch (error) {
    console.error("Error opening file:", error);
  }
}
async function openFolder(path: string) {
  try {
    // convert file path to folder path
    const folderPath = path.substring(0, path.lastIndexOf('\\'));
    console.log(folderPath);
    await open(folderPath);
  } catch (error) {
    console.error("Error opening folder:", error);
  }
}

function getFileIcon(result: SearchResult): string {
  if (result.is_dir) {
    return "📁";
  }
  const ext = result.path.split('.').pop()?.toLowerCase() || "";
  switch (ext) {
    case "txt": return "📄";
    case "pdf": return "📑";
    case "doc":
    case "docx": return "📝";
    case "xls":
    case "xlsx": return "📊";
    case "jpg":
    case "jpeg":
    case "png":
    case "gif": return "🖼️";
    case "mp3":
    case "wav": return "🎵";
    case "mp4":
    case "avi":
    case "mov": return "🎬";
    case "zip":
    case "rar":
    case "7z": return "📦";
    case "exe":
    case "msi": return "⚙️";
    default: return "📄";
  }
}

async function closeWindow() {
  const window=await getCurrentWindow();
  await window.close();
}
async function toggleAlwaysOnTop() {
  const window=await getCurrentWindow();
  isAlwaysOnTop = !isAlwaysOnTop;
  await window.setAlwaysOnTop(isAlwaysOnTop);
}
let unlisten:any;
let unlistenFocus:any;
const init = async () => {
 //监听搜索场景变化
 unlisten = await listen<{ scenarios: SearchScenarios[] }>('search-scenarios-changed', (event) => {
    searchScenarios = [...event.payload.scenarios];
  });
  
 

  //加载已有的搜索场景
  store=await Store.load('finder-settings.json');
  searchScenarios=await store.get<SearchScenarios[]>('searchScenarios')??[];
  //默认搜索场景加载
  const lastss=await store.get<string>('lastSearchState');
  if (lastss) {
    const state=searchScenarios.find(s=>s.name===lastss);
    if (state) {
      currentSearchState=state;
    }
  }else{
    currentSearchState=searchScenarios[0]??undefined;
  }
  usedPath=await store.get<string[]>('usedPath');
//加载全局快捷键
globalShortcutData = await store.get<string>("globalShortcut");
    // await unregisterAll();
    if (globalShortcutData) {
      const isAlreadyRegistered=await isRegistered(globalShortcutData);
      if (!isAlreadyRegistered) {
        await register(globalShortcutData, async() => {
          const window = getCurrentWindow();
          window?.show();
          window?.setFocus();
          search_input?.focus();
        });
      }
    }
};
let search_input:HTMLInputElement|null=$state<HTMLInputElement|null>(null);
onMount(()=>{
  search_input?.focus();
  return ()=>{
    unlisten();
    unlistenFocus?.();
    unregisterAll();
  }
})

$effect(() => {
  if (searchResults.length > 0 && !isSearching) {
    selectedIndex = 0;
    setTimeout(() => {
      resultsContainer?.focus();
      scrollToSelected();
    }, 100); // 增加延时确保DOM渲染完成
  }
});

const handleKeydown =async (event: KeyboardEvent) => {
  if (!searchResults.length) return;
  
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex = (selectedIndex + 1) % searchResults.length;
      scrollToSelected();
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex = (selectedIndex - 1 + searchResults.length) % searchResults.length;
      scrollToSelected();
      break;
    case 'Enter':
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < searchResults.length) {
        if (event.ctrlKey) {
          await openFolder(searchResults[selectedIndex].path);
        } else {
          await openFile(searchResults[selectedIndex].path);
        }
      }
      break;
  }
};

const scrollToSelected = () => {
  const selectedElement = resultsContainer?.querySelector('.selected') as HTMLElement;
  if (selectedElement) {
    selectedElement.scrollIntoView({ block: 'nearest' });
  }
};
</script>
<div data-tauri-drag-region class="titlebar">
  <button class="titlebar-button" onclick={async()=>await message('Created by Patrick,just for easy working, have fun', { title: 'Finder V0.1.3', kind: 'info' })} aria-label="brand">

    <img class="brand" width="20px" height="20px" src="/brand.png" alt="brand"/>
  </button>
  <div class="left-top">

    <button onclick={toggleAlwaysOnTop} aria-label="alwaysontop" class="titlebar-button" id="titlebar-ontop">
      <svg
      class="top-icon"
        class:ontop={isAlwaysOnTop}
        viewBox="0 0 1024 1024"
        version="1.1"
        xmlns="http://www.w3.org/2000/svg"
        width="200"
        height="200"
        ><path
        d="M931.2 276.5l-211.4-190c-15.7-14.1-35.3-21-54.8-21-22.5 0-44.8 9.2-61 27.2L434.6 281.2c-10.6 11.8-24.6 20.2-40 23.9l-229.6 56c-56.1 13.7-73.9 84.5-31 123.1l201.8 181.3L70.2 931.1c-6.2 6.2-6.2 16.4 0 22.6 3.1 3.1 7.2 4.7 11.3 4.7s8.2-1.6 11.3-4.7L359.6 687l238.5 214.3c14.1 12.7 31 18.5 47.6 18.5 33.8 0 66.4-24 71.5-62.4l31.2-234.3c2.1-15.8 8.9-30.5 19.5-42.4l169.4-188.5c30.3-33.6 27.6-85.5-6.1-115.7z m-17.6 94.4L744.1 559.4c-15 16.7-24.5 37.3-27.5 59.5l-31.2 234.3c-3 22.7-22.3 34.6-39.8 34.6-6.8 0-16.8-1.8-26.2-10.3L155.3 460.4c-11.2-10-15.6-24.7-12-39.3 3.7-14.6 14.6-25.4 29.2-28.9l229.6-56c21.8-5.3 41.3-16.9 56.3-33.6l169.4-188.5c9.5-10.5 23-16.6 37.2-16.6 12.3 0 24.2 4.5 33.4 12.8l211.4 190c20.5 18.4 22.2 50.1 3.8 70.6z"
        ></path></svg
        >
      </button>
      <button aria-label="close" onclick={closeWindow} class="titlebar-button" id="titlebar-close">
        <svg class="top-icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" width="200" height="200"><path d="M568 515.008l254.016-255.008q12-11.008 12-27.488t-11.488-28-28-11.488-27.488 12l-255.008 254.016-255.008-254.016q-11.008-12-27.488-12t-28 11.488-11.488 28 12 27.488l254.016 255.008-254.016 255.008q-12 11.008-12 27.488t11.488 28 28 11.488 27.488-12l255.008-255.008 255.008 255.008q11.008 12 27.488 12t28-11.488 11.488-28-12-27.488z"></path></svg>
      </button>
    </div>
</div>
<main class="container">
  {#await init()}
    loading...
  {:then _} 
  <div class="search-container">
    <div class="input-wrapper">
      <input
        bind:this={search_input}
        type="text"
        class="search-input"
        bind:value={searchTerm}
        placeholder="type here..."
        onkeydown={handleKeyDown}
      />
      {#if searchTerm}
        <button class="clear-button" aria-label="clear" onclick={clearSearch}>
          <svg class="icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" width="200" height="200"><path d="M822.00345 776.822434l0.022513-0.022513L246.50423 201.317075c-5.78782-5.791913-13.785981-9.374508-22.621207-9.374508-17.662265 0-31.980365 14.3181-31.980365 31.980365 0 8.834202 3.582595 16.832364 9.373485 22.620184L776.11226 821.339324c5.838985 6.277984 14.166651 10.209526 23.416316 10.209526 17.662265 0 31.980365-14.3181 31.980365-31.980365C831.508941 790.667767 827.871087 782.620487 822.00345 776.822434z" p-id="1472"></path><path d="M776.783549 201.448058l-0.022513-0.022513L201.278189 776.947278c-5.791913 5.78782-9.374508 13.785981-9.374508 22.621207 0 17.662265 14.3181 31.980365 31.980365 31.980365 8.834202 0 16.832364-3.582595 22.620184-9.373485l574.797231-574.836117c6.277984-5.838985 10.209526-14.166651 10.209526-23.416316 0-17.662265-14.3181-31.980365-31.980365-31.980365C790.628882 191.942567 782.580578 195.58042 776.783549 201.448058z"></path></svg>
        </button>
      {/if}
      <button 
        class="search-btn" 
        onclick={search}
      >
        {#if isSearching}
          <div ></div>
        {:else}
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        {/if}
      </button>
    </div>
    <div class="dropdown">
      <button 
        class="dropdown-btn" 
        onclick={() => (isDropdownOpen = !isDropdownOpen)}
        aria-label="search states"
      >
        <span class="current-state">{currentSearchState?.name||'...'}</span>
        <svg class="dropdown-icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg" width="200" height="200">
          <path
            d="M512 714.666667c-8.533333 0-17.066667-2.133333-23.466667-8.533334l-341.333333-341.333333c-12.8-12.8-32-12.8-44.8 0l320 320 320-320c12.8-12.8 12.8-32 0-44.8l-341.333333 341.333333c-6.4 6.4-14.933333 8.533333-23.466667 8.533334z"></path>
        </svg>
      </button>
      {#if isDropdownOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="dropdown-menu"  aria-label="search states" onclick={e=>e.stopPropagation()}>
          {#each searchScenarios as state}
            <button 
              class="dropdown-item" 
              class:active={state.name === currentSearchState?.name}
              onclick={() => selectSearchState(state)}
            >
              {state.name}
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
  <div 
    class="results-container" 
    bind:this={resultsContainer}
    tabindex="-1"
    onkeydown={handleKeydown}
  >
  
    {#if searchResults.length > 0}
    {#each searchResults as result, i}
      <div aria-label="result-item" title={result.path}
        class="result-item" 
        class:selected={i === selectedIndex}
        onclick={() => {
          selectedIndex = i;
        }}
        ondblclick={async () => {
          selectedIndex = i;
          await openFile(result.path);
        }}
      >

        <div class="file-icon">
          {@html getFileIcon(result)}
        </div>
        <div class="file-name">
          {result.name}
        </div>
    
    </div>
    {/each}
  {:else if isSearching}
    <div class="searching">
      <button class="cancel-btn" title="Cancel search" onclick={async(e)=>{await cancelSearch();e.stopPropagation()}}>
        Cancel
        <div class="spinner"></div>
      </button>
      </div>
      {:else}
      <div class="hints">

        <div>Press <strong>Enter</strong> to open result item</div>
        <div>Press <strong>Ctrl + Enter</strong> to open the folder of result item located in </div>
      </div>
  {/if}
  </div>
  {/await}
  
</main>

<style>

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
  overflow: hidden;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}
.titlebar {
  height: 30px;
  background: transparent;
  user-select: none;
  display: flex;
  justify-content:space-between;
  position: fixed;
  top: 2px;
  left: 4px;
  right: 8px;
}
.left-top{
  margin: 0;
  padding: 0;
}
.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
  user-select: none;
  -webkit-user-select: none;
  border:none;
  background-color: transparent;
}
.titlebar-button:hover {
  scale: 1.1;
  cursor: pointer;
}
.top-icon {
  width: 20px;
  height: 20px;
  fill: #5bbec3;
}
.ontop{
  fill: #bc9191;
  background: #addede;
}
.container {
  margin-top: 30px;
  padding: 1rem 1rem 0rem 1rem; 
  
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}
.search-container {
  width: 100%;
  display: flex;
  justify-content: start;
  align-items: center;
  gap:0.5rem;
}
.input-wrapper {
  flex:1;
  position: relative;
  display: flex;
}
.search-input {
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 8px;
  width: 100%;
  font-size: 14px;
  box-sizing: border-box;
  outline: none;
  color:#666;
  overflow: hidden;
}
.search-btn {
  position: absolute;
  top: 50%;
  right: 0;
  transform: translateY(-50%);
  border: none;
  outline: none;
  cursor: pointer;
  background: transparent;
  color: #666;
    padding: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
}
.search-btn:hover {
  color: #5bbec3;
  scale: 1.1;
}
.spinner {
    width: 20px;
    height: 20px;
    border: 2px solid #f3f3f3;
    border-top: 2px solid #5bbec3;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
.clear-button {
  position: absolute;
  top: 50%;
  right: 2rem;
  transform: translateY(-50%);
  border: none;
  outline: none;
  color: #ced4da;
  background: transparent;
  cursor: pointer;
  padding: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
}
.clear-button:hover {
  color: #5bbec3;
  scale: 1.1;
}
.dropdown {
  position: relative;
  display: inline-block;
}
.dropdown-btn {
  border: 1px solid #ccc;
  border-radius: 4px;
  width: 6rem;
  padding: 8px 6px;
  font-size: 16px;
  background-color: #5bbec3;
  color: #fff;
  cursor: pointer;
  transition: background-color 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: space-between
}
.dropdown-btn:hover {
  background-color: #5bbec3;
}
.current-state {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 0.5rem;
}
.dropdown-icon {
  width: 16px;
  height: 16px;
  fill: #fff;
  transition: transform 0.3s ease;
}
.dropdown:hover .dropdown-icon {
  transform: rotate(180deg);
}
.dropdown-menu {
    position: absolute;
    top: calc(100% + 5px);
    right: 0;
    background: white;
    border: 1px solid var(--border-color);
    border-radius: 0.4rem;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    z-index: 1000;
    min-width: 150px;
    max-height: 300px;
    overflow-y: auto;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 0.5rem 1rem;
    text-align: left;
    border: none;
    background: transparent;
    cursor: pointer;
    color: #333;
  }

  .dropdown-item:hover {
    background: #f8f9fa;
  }

  .dropdown-item.active {
    background: #e9ecef;
    color: #007bff;
  }
.results-container {
  margin-top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow-y: auto;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 8px;
  height: 300px;
}
.results-container:focus {
  outline: none;
}
.result-item {
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 8px 24px 8px 8px;
  text-align: left;
  transition: background-color 0.3s ease;  
  display: flex;
  align-items: center;
  gap: 0.5rem;
  position: relative;
}
.result-item:hover {
  background-color: #5bbec3;
  color: #fff;
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.selected {
  background-color: #5bbec3;
  color: #fff;
}
.file-icon {
  width: 24px;
  height: 24px;
  fill: #5bbec3;
}
.searching{
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
}
.cancel-btn {
  border: none; 
  /* background: #b6b4b4; */
  border-radius: 0.3rem;
  width: 6rem;
  height: 2rem;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  color:rgb(185, 178, 178);
}
.cancel-btn:hover {
  color: white;
  background: #c2c5c5;
  scale: 1.1;
}
.hints{
  position: absolute;
  width: 80%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 1rem;
  flex-direction: column;
  color:rgb(158, 154, 154)
}
</style>
