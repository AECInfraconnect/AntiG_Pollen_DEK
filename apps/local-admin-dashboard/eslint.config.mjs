import js from "@eslint/js";

export default [
    js.configs.recommended,
    {
        files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
        rules: {
            "no-unused-vars": "warn",
            "no-undef": "off", 
        }
    }
];
