import { readdir, stat } from "node:fs/promises";
import path from "node:path";
import { spawn } from "node:child_process";
import os from "node:os";

async function walk(dir) {
	const results = [];
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const fullPath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			results.push(fullPath);
			results.push(...(await walk(fullPath)));
		} else {
			results.push(fullPath);
		}
	}
	return results;
}

async function findBundle() {
	const searchRoots = [
		"target/dx",
		"dist",
		"crates/app-desktop/dist",
		"target/release",
	];
	for (const root of searchRoots) {
		try {
			const entries = await walk(root);
			const appBundle = entries.find((entry) => entry.endsWith(".app"));
			if (appBundle) return { kind: "app", path: appBundle };

			const binary = entries.find((entry) =>
				/app-desktop|pig-studio/.test(path.basename(entry)),
			);
			if (binary) return { kind: "bin", path: binary };
		} catch {
			// ignore missing roots
		}
	}
	throw new Error(
		"No bundled app or binary found under target/dx, dist, crates/app-desktop/dist, or target/release",
	);
}

async function macExecutable(appPath) {
	const macOsDir = path.join(appPath, "Contents", "MacOS");
	const entries = await readdir(macOsDir);
	if (entries.length === 0) {
		throw new Error(`No executable found inside ${macOsDir}`);
	}
	return path.join(macOsDir, entries[0]);
}

async function ensureCssBundled(bundlePath) {
	const entries = await walk(bundlePath);
	const hasCss = entries.some((entry) => entry.endsWith("generated.css"));
	if (!hasCss)
		throw new Error("generated.css was not found in the bundled output");
}

async function launchAndSmoke(binaryPath) {
	const cwd = await stat(os.tmpdir()).then(() => os.tmpdir());
	const child = spawn(binaryPath, [], {
		cwd,
		stdio: "ignore",
		detached: true,
	});

	let exitedEarly = false;
	child.on("exit", () => {
		exitedEarly = true;
	});

	await new Promise((resolve) => setTimeout(resolve, 3000));

	try {
		process.kill(-child.pid, "SIGTERM");
	} catch {
		// ignore cleanup failures
	}

	if (exitedEarly) {
		throw new Error("Bundled app exited before smoke check completed");
	}
}

const bundle = await findBundle();
const bundleRoot = path.resolve(
	bundle.kind === "app" ? bundle.path : path.dirname(bundle.path),
);
await ensureCssBundled(bundleRoot);
const executable = path.resolve(
	bundle.kind === "app" ? await macExecutable(bundle.path) : bundle.path,
);
await launchAndSmoke(executable);
console.log(`Bundle smoke check passed: ${executable}`);
