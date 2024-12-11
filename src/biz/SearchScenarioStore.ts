import { writable } from "svelte/store";
import type { SearchScenarios } from "./types";

export const SearchScenarioStore = writable<SearchScenarios[]>([]);

export const resetSearchScenarios = () => SearchScenarioStore.set([]);

export const setSearchScenarios = (searchScenarios: SearchScenarios[]) => SearchScenarioStore.set(searchScenarios);

export const addSearchScenario = (searchScenario: SearchScenarios) => SearchScenarioStore.update((scenarios) => [...scenarios, searchScenario]);

export const updateSearchScenario = (index: number, searchScenario: SearchScenarios) => SearchScenarioStore.update((scenarios) => {
    scenarios[index] = searchScenario;
    return [...scenarios];
});

export const removeSearchScenario = (index: number) => SearchScenarioStore.update((scenarios) => {
    scenarios.splice(index, 1);
    return [...scenarios];
});