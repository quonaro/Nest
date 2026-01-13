
import * as path from 'path';
import * as fs from 'fs';

// Mock vscode module
const vscodeMock = require('./vscode_mock');

// Hijack module resolution to serve our mock when 'vscode' is requested
const originalRequire = require('module').prototype.require;
require('module').prototype.require = function (id: string) {
    if (id === 'vscode') {
        return vscodeMock;
    }
    return originalRequire.call(this, id);
};

// Now import the validator
// We need to use ts-node to run this, assuming ts-node is available or we compile it.
// Since we are likely running with ts-node, we can import directly.
import { validateNestfileDocument } from '../src/validator';

// Setup test case
const nestfilePath = path.resolve(__dirname, '../Nestfile');
const nestfileContent = `@include "./nestfile" into nesty
@var APP_NAME = "vscode-nestfile-support"
`;

console.log(`Testing validation on file: ${nestfilePath}`);
console.log(`Content:\n${nestfileContent}`);

// Create a mock URI
const docUri = vscodeMock.Uri.file(nestfilePath);

// Run validation
const diagnostics = validateNestfileDocument(nestfileContent, {}, docUri);

// Check results
console.log(`Found ${diagnostics.length} diagnostics:`);
let foundError = false;
for (const diag of diagnostics) {
    console.log(`- [${diag.severity}] Line ${diag.range.start.line}: ${diag.message}`);
    if (diag.message.includes('Include file not found') || diag.message.includes('Include path not found')) {
        foundError = true;
    }
}

if (foundError) {
    console.log("\nFAIL: Reproduced 'Include file not found' error.");
    process.exit(1);
} else {
    console.log("\nSUCCESS: No include errors found.");
    process.exit(0);
}
