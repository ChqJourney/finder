<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import type { SearchScenarios } from "../../biz/types";
    import { onMount } from "svelte";
    import { Store } from "@tauri-apps/plugin-store";
    import { emit } from '@tauri-apps/api/event';

    let store:Store;
    let activeTab=$state<string>('search');
    let searchScenarios=$state<SearchScenarios[]>([]);
    let newSearchScenario=$state<SearchScenarios>({
        name: '',
        path: '',
        level: 'top',
        target: 'files',
        fileExtensions: ''
    });
    let formErrors: {name?: string, path?: string, fileExtensions?: string} = $state({});
    let globalShortcut = $state<string>('');
    let isCapturingShortcut = $state<boolean>(false);

    const closeWindow = async () => {
        const window=await getCurrentWindow();
        await window.close();
    }
    const validateSearchStateName = (name: string, searchStates: SearchScenarios[]): string | undefined => {
        if (name.trim() === '') {
            return 'Search state name cannot be empty';
        }
        const existingState = searchStates.find(state => state.name === name);
        return existingState ? 'Search state name already exists' : undefined;
    };
    const validateWindowsPath = (path: string): string | undefined => {
        if (path.trim() === '') {
            return 'Path cannot be empty';
        }
        // Windows path validation for both local and UNC network paths
        const localPathRegex = /^[a-zA-Z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/;
        const uncPathRegex = /^\\\\[^\\/:*?"<>|\r\n]+\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/;
        
        if (!localPathRegex.test(path) && !uncPathRegex.test(path)) {
            return 'Invalid Windows path format';
        }
        return undefined;
    };
    const validateFileExtensions = (extensions: string): string | undefined => {
        // Empty string is valid
        if (extensions.trim() === '') {
            return undefined;
        }

        // Remove trailing comma if exists
        let cleanExtensions = extensions;
        if (cleanExtensions.endsWith(',')) {
            cleanExtensions = cleanExtensions.slice(0, -1);
        }

        // Split by comma
        const extensionsArray = cleanExtensions.split(',');

        for (const extension of extensionsArray) {
            const trimmedExt = extension.trim();
            
            // Check for empty extension between commas
            if (trimmedExt === '') {
                return 'File extension cannot be empty';
            }

            // Check for any non-alphanumeric characters except dots
            if (!trimmedExt.match(/^[a-zA-Z0-9.]*$/)) {
                return 'Invalid file extension format';
            }
        }

        return undefined;
    };
    const addNew = async () => {
        // Reset errors
        formErrors = {};
        
        // Validate name
        const nameError = validateSearchStateName(newSearchScenario.name, searchScenarios);
        if (nameError) {
            formErrors.name = nameError;
        }

        // Validate path
        const pathError = validateWindowsPath(newSearchScenario.path);
        if (pathError) {
            formErrors.path = pathError;
        }
        // validate file extensions
        const extensionError = validateFileExtensions(newSearchScenario.fileExtensions);
        if (extensionError) {
            formErrors.fileExtensions = extensionError;
        }
        console.log(newSearchScenario);
        if (!formErrors.name && !formErrors.path&& !formErrors.fileExtensions) {
        searchScenarios.push(newSearchScenario);
        newSearchScenario = {
            name: '',
            path: '',
            level: 'top',
            target: 'files',
            fileExtensions: ''
        };
        formErrors = {};
        await emit('search-scenarios-changed', { scenarios: searchScenarios });
        await store.set('searchScenarios', searchScenarios);
        // await store.save();
    }
    };
    const deleteSearchScenario =async (index: number) => {
        searchScenarios.splice(index, 1);
        await emit('search-scenarios-changed', { scenarios: searchScenarios });
        await store.set('searchScenarios', searchScenarios);
        // await store.save();
    };
    const startCapturingShortcut = () => {
        isCapturingShortcut = true;
    };

    const stopCapturingShortcut = () => {
        isCapturingShortcut = false;
    };

    const handleKeyDown = (event: KeyboardEvent) => {
        if (!isCapturingShortcut) return;
        
        event.preventDefault();
        
        const modifiers = [];
        if (event.ctrlKey) modifiers.push('Ctrl');
        if (event.shiftKey) modifiers.push('Shift');
        if (event.altKey) modifiers.push('Alt');
        
        const key = event.key.toUpperCase();
        if (!['CONTROL', 'SHIFT', 'ALT'].includes(key)) {
            globalShortcut = [...modifiers, key].join('+');
            store.set('globalShortcut', globalShortcut);
            // store.save();
            isCapturingShortcut = false;
        }
    };
    const init=async()=>{
        store=await Store.load('finder-settings.json');
       searchScenarios=await store.get<SearchScenarios[]>('searchScenarios')??[];
       globalShortcut = await store.get<string>('globalShortcut') ?? '';
       document.addEventListener('keydown', handleKeyDown);
    }
    onMount(()=>{
       return ()=>{
        document.removeEventListener('keydown', handleKeyDown);
        // unregister(globalShortcut);
       }
    })
</script>
<div data-tauri-drag-region class="titlebar">

    <button aria-label="close" class="close-btn" onclick={closeWindow}>
        <svg class="top-icon" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg"  width="200" height="200"><path d="M822.00345 776.822434l0.022513-0.022513L246.50423 201.317075c-5.78782-5.791913-13.785981-9.374508-22.621207-9.374508-17.662265 0-31.980365 14.3181-31.980365 31.980365 0 8.834202 3.582595 16.832364 9.373485 22.620184L776.11226 821.339324c5.838985 6.277984 14.166651 10.209526 23.416316 10.209526 17.662265 0 31.980365-14.3181 31.980365-31.980365C831.508941 790.667767 827.871087 782.620487 822.00345 776.822434z"></path><path d="M776.783549 201.448058l-0.022513-0.022513L201.278189 776.947278c-5.791913 5.78782-9.374508 13.785981-9.374508 22.621207 0 17.662265 14.3181 31.980365 31.980365 31.980365 8.834202 0 16.832364-3.582595 22.620184-9.373485l574.797231-574.836117c6.277984-5.838985 10.209526-14.166651 10.209526-23.416316 0-17.662265-14.3181-31.980365-31.980365-31.980365C790.628882 191.942567 782.580578 195.58042 776.783549 201.448058z"></path></svg>
    </button>
</div>

<div class="container">
    {#await init()}
        loading...
    {:then _} 
    <div class="tabs">
        <button 
            class="tab-btn" 
            class:active={activeTab === 'search'} 
            onclick={() => activeTab = 'search'}>
            Search Settings
        </button>
        <button 
            class="tab-btn" 
            class:active={activeTab === 'others'} 
            onclick={() => activeTab = 'others'}>
            Others
        </button>
        
    </div>

    <div class="content-wrapper">
        {#if activeTab === 'search'}
        <h3 >Search Scenarios</h3>
        
        <!-- Add new search state form -->
        <div class="form-container">
            <div class="form-group">
                <div class="input-row">
                    <div class="input-group">
                        <label for="search-name">Search State Name</label>
                        <input 
                            id="search-name"
                            type="text" 
                            bind:value={newSearchScenario.name} 
                            placeholder="Enter name..."
                            class:error={formErrors.name}
                        />
                        {#if formErrors.name}
                            <div class="error-message">{formErrors.name}</div>
                        {/if}
                    </div>
                    <div class="input-group">
                        <label for="search-path">Search Path</label>
                        <input 
                            id="search-path"
                            type="text" 
                            bind:value={newSearchScenario.path} 
                            placeholder="Enter path..."
                            class:error={formErrors.path}
                        />
                        {#if formErrors.path}
                            <div class="error-message">{formErrors.path}</div>
                        {/if}
                    </div>
                </div>
                
                <div class="select-row">
                    <div class="input-group">
                        <label for="search-level">Search Level</label>
                        <select id="search-level" bind:value={newSearchScenario.level}>
                            <option value="top">Top level only</option>
                            <option value="all">All subfolders</option>
                        </select>
                    </div>
                    
                    <div class="input-group">
                        <label for="search-target">Search Target</label>
                        <select id="search-target" bind:value={newSearchScenario.target}>
                            <option value="both">Files & Folders</option>
                            <option value="files">Files Only</option>
                            <option value="folders">Folders Only</option>
                        </select>
                    </div>
                </div>
                
                <div class="input-group">
                    <label for="file-extensions">File Extensions</label>
                    <input disabled={newSearchScenario.target === 'folders'}
                        id="file-extensions"
                        type="text" 
                        bind:value={newSearchScenario.fileExtensions} 
                        placeholder="e.g. jpg, png, pdf"
                        class:error={formErrors.fileExtensions}
                    />
                    {#if formErrors.fileExtensions}
                        <div class="error-message">{formErrors.fileExtensions}</div>
                    {/if}
                </div>
                
                <button class="add-btn" onclick={addNew}>Add Search State</button>
            </div>
        </div>

        <!-- Search scenarios table -->
        <div class="table-container">
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Path</th>
                        <th>Level</th>
                        <th>Target</th>
                        <th>File Extension</th>
                        <th>Action</th>
                    </tr>
                </thead>
                <tbody>
                    {#each searchScenarios as scenario, i}
                        <tr>
                            <td>{scenario.name}</td>
                            <td>{scenario.path}</td>
                            <td>{scenario.level}</td>
                            <td>{scenario.target}</td>
                            <td>{scenario.fileExtensions}</td>
                            <td>
                                <button aria-label="Delete" class="table-delete-btn" onclick={() => deleteSearchScenario(i)}>
                                    <svg viewBox="0 0 24 24" width="16" height="16">
                                        <path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12 19 6.41z"/>
                                    </svg>
                                </button>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
        {:else if activeTab === 'others'}
        <h3>Other settings</h3>
        <div class="shortcut-section">
            <h3>Global Shortcut</h3>
            <div class="shortcut-container">
                <div class="shortcut-display" class:capturing={isCapturingShortcut}>
                    {#if isCapturingShortcut}
                        Press your desired shortcut...
                    {:else}
                        {globalShortcut || 'No shortcut set'}
                    {/if}
                </div>
                <button 
                    class="shortcut-btn"
                    onclick={isCapturingShortcut ? stopCapturingShortcut : startCapturingShortcut}
                >
                    {isCapturingShortcut ? 'Cancel' : 'Set Shortcut'}
                </button>
                <div class="shortcut-hint">p.s. 重启后生效</div>
            </div>
        </div>
        {:else}
        error
        {/if}
        </div>
    {/await}
    
</div>
<style>
    .shortcut-hint{
        color: red;
        font-size: 14px;
    }
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
  justify-content: flex-end;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
}
    .container {
        padding: 0.5rem 1rem;
        height: 100vh;
        width: 100%;
        box-sizing: border-box;
        display: flex;
        flex-direction: column;
    }

    .close-btn {
        position: fixed;
        top: 0.5rem;
        right: 0.5rem;
        background: transparent;
        border: none;
        cursor: pointer;
        padding: 0.2rem 0.3rem;
        transition: transform 0.2s;
        z-index: 10;
    }

    .close-btn:hover {
        transform: scale(1.1);
    }

        .top-icon {
        width: 1.5rem;
        height: 1.5rem;
        fill: #242424;
        opacity: 0.8;
    }

        .top-icon:hover {
        opacity: 1;
    }

    .tabs {
        display: flex;
        gap: 1rem;
        margin-bottom: 1rem;
        border-bottom: 1px solid #333;
        padding-bottom: 0.5rem;
        flex-shrink: 0;
    }

    .tab-btn {
        background: transparent;
        border: none;
        color: #888;
        padding: 0.75rem 1.5rem;
        cursor: pointer;
        font-size: 1rem;
        transition: all 0.2s;
        border-radius: 4px;
        user-select: none;
    }

    .tab-btn:hover {
        color: #4c4b4b;
        background: rgba(255, 255, 255, 0.1);
    }

    .tab-btn.active {
        color: #fff;
        background: #2563eb;
    }

    .content-wrapper {
        padding: 1rem 1rem;
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        flex: 1;
        overflow: auto;
        width: 100%;
        box-sizing: border-box;
        min-height: 0;
    }

    .form-container {
        background: #ffffff;
        border-radius: 8px;
        padding: 1.5rem;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .input-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .select-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        position: relative;
    }

    label {
        font-size: 0.875rem;
        color: #4b5563;
        font-weight: 500;
    }

    input, select {
        padding: 0.625rem;
        border: 1px solid #e5e7eb;
        border-radius: 6px;
        font-size: 0.875rem;
        color: #1f2937;
        background: #f9fafb;
        transition: all 0.2s ease;
    }

    input:hover, select:hover {
        border-color: #d1d5db;
    }

    input:focus, select:focus {
        outline: none;
        border-color: #2563eb;
        box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.1);
    }

    input::placeholder {
        color: #9ca3af;
    }

    input.error {
        border-color: #ef4444;
    }

    .error-message {
        color: #ff4444;
        font-size: 0.75rem;
        position: absolute;
        bottom: -1.25rem;
        left: 0.5rem;
        position: absolute;
    }

    .add-btn {
        background: #2563eb;
        color: white;
        border: none;
        padding: 0.75rem 1.5rem;
        border-radius: 6px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s ease;
        width: fit-content;
        align-self: flex-end;
    }

    .add-btn:hover {
        background: #1d4ed8;
        transform: translateY(-1px);
    }

    .add-btn:active {
        transform: translateY(0);
    }

    .table-container {
        margin-top: 2rem;
        border-radius: 8px;
        background: white;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        overflow: hidden;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.9rem;
    }

    th {
        background: #f8f9fa;
        padding: 12px 16px;
        text-align: left;
        font-weight: 600;
        color: #495057;
        border-bottom: 2px solid #e9ecef;
    }

    td {
        padding: 12px 16px;
        border-bottom: 1px solid #e9ecef;
        color: #212529;
    }

    tbody tr:hover {
        background-color: #f8f9fa;
    }

    .table-delete-btn {
        background: none;
        border: none;
        color: #dc3545;
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background-color 0.2s;
    }

    .table-delete-btn:hover {
        background-color: #fff5f5;
    }

    .table-delete-btn:active {
        background-color: #ffe3e3;
    }

    .shortcut-section {
        padding: 1rem;
        background: #ffffff;
        border-radius: 8px;
        margin-bottom: 1rem;
    }

    .shortcut-container {
        display: flex;
        gap: 1rem;
        align-items: center;
        margin-top: 0.5rem;
    }

    .shortcut-display {
        padding: 0.5rem 1rem;
        border: 1px solid #e0e0e0;
        border-radius: 4px;
        min-width: 200px;
        text-align: center;
        background: #f5f5f5;
    }

    .shortcut-display.capturing {
        border-color: #2196f3;
        background: #e3f2fd;
    }

    .shortcut-btn {
        padding: 0.5rem 1rem;
        background: #2196f3;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
    }

    .shortcut-btn:hover {
        background: #1976d2;
    }
</style>