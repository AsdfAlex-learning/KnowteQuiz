import js from '@eslint/js';
import vue from 'eslint-plugin-vue';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['dist/**', 'node_modules/**', 'src-tauri/**', 'coverage/**', 'scripts/**'],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        sourceType: 'module',
        ecmaVersion: 'latest',
        globals: { ...globals.browser },
      },
    },
  },
  {
    languageOptions: {
      globals: { ...globals.browser },
    },
    rules: {
      // Component naming: App.vue and single-word view containers are fine.
      'vue/multi-word-component-names': 'off',
      // v-html is used only for our own markdown pipelines (html:false,
      // escaped fallbacks), never raw user input.
      'vue/no-v-html': 'off',
      // Formatting is owned by Prettier; these vue rules would fight it.
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
      'vue/html-indent': 'off',
      'vue/html-closing-bracket-newline': 'off',
      'vue/first-attribute-linebreak': 'off',
    },
  },
  {
    files: ['*.cjs'],
    languageOptions: {
      globals: { ...globals.node },
    },
    rules: {
      // CommonJS launch scripts: require() is the point.
      '@typescript-eslint/no-require-imports': 'off',
    },
  }
);
