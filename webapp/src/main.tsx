import type { RouteSectionProps } from "@solidjs/router";
import { render } from "@solidjs/web";
import App from "./App";
import { Router } from "./router";
import "./index.css";

const root = document.getElementById("root");
if (!root) throw new Error("Missing application root");
render(() => <Router>{(props: RouteSectionProps) => <App>{props.children}</App>}</Router>, root);
