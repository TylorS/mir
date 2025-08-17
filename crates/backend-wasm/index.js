const { existsSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const { platform, arch } = process;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

function isMusl() {
	// For Node 10
	if (!process.report || typeof process.report.getReport !== "function") {
		try {
			const lddPath = require("node:child_process")
				.execSync("which ldd")
				.toString()
				.trim();
			return readFileSync(lddPath, "utf8").includes("musl");
		} catch (e) {
			return true;
		}
	} else {
		const { glibcVersionRuntime } = process.report.getReport().header;
		return !glibcVersionRuntime;
	}
}

switch (platform) {
	case "android":
		switch (arch) {
			case "arm64":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.android-arm64.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.android-arm64.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-android-arm64");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			case "arm":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.android-arm-eabi.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.android-arm-eabi.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-android-arm-eabi");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			default:
				throw new Error(`Unsupported architecture on Android ${arch}`);
		}
		break;
	case "win32":
		switch (arch) {
			case "x64":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.win32-x64-msvc.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.win32-x64-msvc.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-win32-x64-msvc");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			case "ia32":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.win32-ia32-msvc.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.win32-ia32-msvc.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-win32-ia32-msvc");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			case "arm64":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.win32-arm64-msvc.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.win32-arm64-msvc.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-win32-arm64-msvc");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			default:
				throw new Error(`Unsupported architecture on Windows: ${arch}`);
		}
		break;
	case "darwin":
		localFileExisted = existsSync(
			join(__dirname, "mir-backend-wasm.darwin-universal.node"),
		);
		try {
			if (localFileExisted) {
				nativeBinding = require("./mir-backend-wasm.darwin-universal.node");
			} else {
				nativeBinding = require("@mir/backend-wasm-darwin-universal");
			}
		} catch (e) {
			loadError = e;
		}
		switch (arch) {
			case "x64":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.darwin-x64.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.darwin-x64.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-darwin-x64");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			case "arm64":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.darwin-arm64.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.darwin-arm64.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-darwin-arm64");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			default:
				throw new Error(`Unsupported architecture on macOS: ${arch}`);
		}
		break;
	case "freebsd":
		if (arch !== "x64") {
			throw new Error(`Unsupported architecture on FreeBSD: ${arch}`);
		}
		localFileExisted = existsSync(
			join(__dirname, "mir-backend-wasm.freebsd-x64.node"),
		);
		try {
			if (localFileExisted) {
				nativeBinding = require("./mir-backend-wasm.freebsd-x64.node");
			} else {
				nativeBinding = require("@mir/backend-wasm-freebsd-x64");
			}
		} catch (e) {
			loadError = e;
		}
		break;
	case "linux":
		switch (arch) {
			case "x64":
				if (isMusl()) {
					localFileExisted = existsSync(
						join(__dirname, "mir-backend-wasm.linux-x64-musl.node"),
					);
					try {
						if (localFileExisted) {
							nativeBinding = require("./mir-backend-wasm.linux-x64-musl.node");
						} else {
							nativeBinding = require("@mir/backend-wasm-linux-x64-musl");
						}
					} catch (e) {
						loadError = e;
					}
				} else {
					localFileExisted = existsSync(
						join(__dirname, "mir-backend-wasm.linux-x64-gnu.node"),
					);
					try {
						if (localFileExisted) {
							nativeBinding = require("./mir-backend-wasm.linux-x64-gnu.node");
						} else {
							nativeBinding = require("@mir/backend-wasm-linux-x64-gnu");
						}
					} catch (e) {
						loadError = e;
					}
				}
				break;
			case "arm64":
				if (isMusl()) {
					localFileExisted = existsSync(
						join(__dirname, "mir-backend-wasm.linux-arm64-musl.node"),
					);
					try {
						if (localFileExisted) {
							nativeBinding = require("./mir-backend-wasm.linux-arm64-musl.node");
						} else {
							nativeBinding = require("@mir/backend-wasm-linux-arm64-musl");
						}
					} catch (e) {
						loadError = e;
					}
				} else {
					localFileExisted = existsSync(
						join(__dirname, "mir-backend-wasm.linux-arm64-gnu.node"),
					);
					try {
						if (localFileExisted) {
							nativeBinding = require("./mir-backend-wasm.linux-arm64-gnu.node");
						} else {
							nativeBinding = require("@mir/backend-wasm-linux-arm64-gnu");
						}
					} catch (e) {
						loadError = e;
					}
				}
				break;
			case "arm":
				localFileExisted = existsSync(
					join(__dirname, "mir-backend-wasm.linux-arm-gnueabihf.node"),
				);
				try {
					if (localFileExisted) {
						nativeBinding = require("./mir-backend-wasm.linux-arm-gnueabihf.node");
					} else {
						nativeBinding = require("@mir/backend-wasm-linux-arm-gnueabihf");
					}
				} catch (e) {
					loadError = e;
				}
				break;
			default:
				throw new Error(`Unsupported architecture on Linux: ${arch}`);
		}
		break;
	default:
		throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`);
}

if (!nativeBinding) {
	if (loadError) {
		throw loadError;
	}
	throw new Error("Failed to load native binding");
}

// Export all NAPI bindings
const {
	WasmNapiBindings,
	WasmInstanceManager,
	WasmMemoryManager,
	FFIBridge,
	ValueConverter,
} = nativeBinding;

module.exports = {
	WasmNapiBindings,
	WasmInstanceManager,
	WasmMemoryManager,
	FFIBridge,
	ValueConverter,
};

// Additional convenience exports for better JavaScript experience
module.exports.createWasmRuntime = (config = {}) => {
	const instanceManager = new WasmInstanceManager({
		maxInstances: config.maxInstances || 10,
		memoryLimitMb: config.memoryLimitMb || 64,
		executionTimeoutMs: config.executionTimeoutMs || 5000,
		enableHotReload: config.enableHotReload !== false,
		preserveStateOnReload: config.preserveStateOnReload !== false,
		enableDebugging: config.enableDebugging || false,
	});

	const memoryManager = new WasmMemoryManager();
	const ffiBridge = new FFIBridge();
	const valueConverter = new ValueConverter();
	const napiBindings = new WasmNapiBindings();

	return {
		instanceManager,
		memoryManager,
		ffiBridge,
		valueConverter,
		napiBindings,

		// Convenience methods
		async createInstance(instanceId, wasmBytecode, imports = null) {
			return await instanceManager.createInstance(
				instanceId,
				wasmBytecode,
				imports,
			);
		},

		async executeFunction(instanceId, functionName, args = []) {
			return await instanceManager.executeFunction(
				instanceId,
				functionName,
				args,
			);
		},

		async hotReload(instanceId, newWasmBytecode) {
			return await instanceManager.hotReloadInstance(
				instanceId,
				newWasmBytecode,
			);
		},

		allocateMemory(instanceId, size, type = "buffer") {
			return memoryManager.allocateMemory(instanceId, size, type);
		},

		writeMemory(instanceId, offset, data) {
			return memoryManager.writeMemory(instanceId, offset, data);
		},

		readMemory(instanceId, offset, size) {
			return memoryManager.readMemory(instanceId, offset, size);
		},

		registerHostFunction(config, hostFunction) {
			return ffiBridge.registerHostFunction(config, hostFunction);
		},

		async callHostFunction(functionName, args, instanceId) {
			return await ffiBridge.callHostFunction(functionName, args, instanceId);
		},

		convertValue(sourceValue, _sourceType, targetType) {
			return valueConverter.jsToWasm(sourceValue, targetType);
		},

		getStats() {
			return {
				instances: instanceManager.listInstances(),
				ffi: ffiBridge.getFfiStats(),
				conversions: valueConverter.getConversionStats(),
			};
		},

		cleanup() {
			const instances = instanceManager.listInstances();
			for (const instanceId of instances) {
				instanceManager.removeInstance(instanceId);
			}
		},
	};
};

// Export version information
module.exports.version = require("./package.json").version;
