let client;
let input;
const DEBUG = true /* && $env.DEBUG */
if (DEBUG) {
    console._log = console.log
    console._error = console.error
}

document.addEventListener('DOMContentLoaded', () => {
    client = document.querySelector('canvas')
    input = document.querySelector('input')
    if (DEBUG) {
        console.log = console._log
        console.error = console._error
    }
})
