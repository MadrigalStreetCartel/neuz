let client;
let input;
const DEBUG = true && $env.DEBUG
if (DEBUG) {
    _console = {...console}
}

document.addEventListener('DOMContentLoaded', () => {
    client = document.querySelector('canvas')
    input = document.querySelector('input')
    if (DEBUG) {
        console = {..._console}
    }
})
