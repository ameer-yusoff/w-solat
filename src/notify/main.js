import { mount } from 'svelte';
import '../styles/global.css';
import Notify from './Notify.svelte';

document.addEventListener('contextmenu', (e) => e.preventDefault());

export default mount(Notify, { target: document.getElementById('app') });
