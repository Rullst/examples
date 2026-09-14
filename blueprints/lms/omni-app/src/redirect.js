const backendUrl = "https://rullst-lms.redpond-24d9228d.eastus.azurecontainerapps.io";
const status = document.querySelector("[data-status]");
const retry = document.querySelector("[data-retry]");

function openBackend() {
  retry.disabled = true;
  status.textContent = "Opening the secure web application…";
  window.location.assign(backendUrl);
}

retry.addEventListener("click", openBackend);
window.addEventListener("offline", () => {
  retry.disabled = false;
  status.textContent = "This device is offline. Reconnect and try again.";
});
window.addEventListener("online", openBackend);

if (navigator.onLine) {
  openBackend();
} else {
  retry.disabled = false;
  status.textContent = "This device is offline. Reconnect and try again.";
}
