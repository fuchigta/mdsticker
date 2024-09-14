import React from "react";
import ReactDOM from "react-dom/client";
import TrashBoxApp from "./TrashBoxApp";
import * as hasOwn from "object.hasown";

if (!Object.hasOwn) {
  hasOwn.default.shim();
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TrashBoxApp />
  </React.StrictMode>,
);
