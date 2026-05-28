module.exports = {
  "plugins": ["react"],
  "rules": {
    "no-restricted-syntax": [
      "error",
      {
        "selector": "JSXOpeningElement[name.name=\"svg\"]",
        "message": "Inline SVGs are not allowed. Please use pre-saved SVG files from public/icons instead."
      }
    ]
  }
};