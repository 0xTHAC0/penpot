import { describe, test, expect } from "vitest";
import { TextEditorMock } from "../../test/TextEditorMock.js";
import { SelectionController } from "../controllers/SelectionController.js";
import { paste } from "./paste.js";

/* @vitest-environment jsdom */

/**
 * Creates a minimal `ClipboardEvent`-like object carrying the given data.
 *
 * @param {Object.<string, string>} data
 * @returns {object}
 */
function createClipboardEvent(data) {
  return {
    preventDefault() {},
    clipboardData: {
      types: Object.keys(data),
      getData(type) {
        return data[type] ?? "";
      },
    },
  };
}

/**
 * Creates a minimal `ClipboardEvent`-like object carrying plain text.
 *
 * @param {string} text
 * @returns {object}
 */
function createPlainTextClipboardEvent(text) {
  return createClipboardEvent({ "text/plain": text });
}

describe("paste", () => {
  test("should insert plain text into an empty editor that was just focused", () => {
    const textEditorMock = TextEditorMock.createTextEditorMockWithText("");
    const selection = document.getSelection();
    const selectionController = new SelectionController(
      textEditorMock,
      selection,
    );
    textEditorMock.element.focus();

    paste(
      createPlainTextClipboardEvent("Hello, World!"),
      textEditorMock,
      selectionController,
    );

    expect(textEditorMock.root.textContent).toBe("Hello, World!");
  });

  test("should insert plain text when the caret is on a paragraph element", () => {
    const textEditorMock =
      TextEditorMock.createTextEditorMockWithText("Hello, ");
    const root = textEditorMock.root;
    const paragraph = root.firstChild;
    const selection = document.getSelection();
    const selectionController = new SelectionController(
      textEditorMock,
      selection,
    );
    textEditorMock.element.focus();
    selection.setBaseAndExtent(paragraph, 1, paragraph, 1);
    document.dispatchEvent(new Event("selectionchange"));

    paste(
      createPlainTextClipboardEvent("World!"),
      textEditorMock,
      selectionController,
    );

    expect(root.textContent).toBe("Hello, World!");
  });

  test("should insert the HTML contents when the editor allows HTML paste", () => {
    const textEditorMock = TextEditorMock.createTextEditorMockWithText("");
    textEditorMock.options = { allowHTMLPaste: true };
    const selection = document.getSelection();
    const selectionController = new SelectionController(
      textEditorMock,
      selection,
    );
    textEditorMock.element.focus();

    paste(
      createClipboardEvent({
        "text/html": "<div>Hello, <b>World!</b></div>",
        "text/plain": "ignored",
      }),
      textEditorMock,
      selectionController,
    );

    expect(textEditorMock.root.textContent).toBe("Hello, World!");
  });

  test("should fall back to plain text when the editor allows HTML paste but there is no HTML", () => {
    const textEditorMock = TextEditorMock.createTextEditorMockWithText("");
    textEditorMock.options = { allowHTMLPaste: true };
    const selection = document.getSelection();
    const selectionController = new SelectionController(
      textEditorMock,
      selection,
    );
    textEditorMock.element.focus();

    paste(
      createPlainTextClipboardEvent("Hello, World!"),
      textEditorMock,
      selectionController,
    );

    expect(textEditorMock.root.textContent).toBe("Hello, World!");
  });
});
