import { writable } from 'svelte/store';

// TODO: figure out how to group these? if that's idiomatic?
export const signedIn = writable(false);
export const userId = writable(-1);
