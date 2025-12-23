<template>
  <Layout>
    <div class="vscode-extension">
      <h1>{{ $t('vscodeExtension.title') }}</h1>
      
      <section id="overview">
        <h2>{{ $t('vscodeExtension.overview.title') }}</h2>
        <p>{{ $t('vscodeExtension.overview.desc') }}</p>
      </section>

      <section id="features">
        <h2>{{ $t('vscodeExtension.features.title') }}</h2>
        <ul>
          <li><strong>{{ $t('vscodeExtension.features.syntaxHighlighting') }}</strong> - {{ $t('vscodeExtension.features.syntaxHighlightingDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.validation') }}</strong> - {{ $t('vscodeExtension.features.validationDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.autocompletion') }}</strong> - {{ $t('vscodeExtension.features.autocompletionDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.hover') }}</strong> - {{ $t('vscodeExtension.features.hoverDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.goToDefinition') }}</strong> - {{ $t('vscodeExtension.features.goToDefinitionDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.findReferences') }}</strong> - {{ $t('vscodeExtension.features.findReferencesDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.codeActions') }}</strong> - {{ $t('vscodeExtension.features.codeActionsDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.formatting') }}</strong> - {{ $t('vscodeExtension.features.formattingDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.snippets') }}</strong> - {{ $t('vscodeExtension.features.snippetsDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.features.codeLens') }}</strong> - {{ $t('vscodeExtension.features.codeLensDesc') }}</li>
        </ul>
      </section>

      <section id="installation">
        <h2>{{ $t('vscodeExtension.installation.title') }}</h2>
        
        <h3>{{ $t('vscodeExtension.installation.marketplace') }}</h3>
        <ol>
          <li>{{ $t('vscodeExtension.installation.marketplaceStep1') }}</li>
          <li>{{ $t('vscodeExtension.installation.marketplaceStep2') }}</li>
          <li>{{ $t('vscodeExtension.installation.marketplaceStep3') }}</li>
        </ol>

        <h3>{{ $t('vscodeExtension.installation.vsix') }}</h3>
        <ol>
          <li>{{ $t('vscodeExtension.installation.vsixStep1') }}</li>
          <li>{{ $t('vscodeExtension.installation.vsixStep2') }}</li>
          <li>{{ $t('vscodeExtension.installation.vsixStep3') }}</li>
          <li>{{ $t('vscodeExtension.installation.vsixStep4') }}</li>
        </ol>
      </section>

      <section id="usage">
        <h2>{{ $t('vscodeExtension.usage.title') }}</h2>
        
        <h3>{{ $t('vscodeExtension.usage.syntaxHighlighting') }}</h3>
        <p>{{ $t('vscodeExtension.usage.syntaxHighlightingDesc') }}</p>

        <h3>{{ $t('vscodeExtension.usage.autocompletion') }}</h3>
        <p>{{ $t('vscodeExtension.usage.autocompletionDesc') }}</p>
        <ul>
          <li>{{ $t('vscodeExtension.usage.autocompletionDirectives') }}</li>
          <li>{{ $t('vscodeExtension.usage.autocompletionMeta') }}</li>
          <li>{{ $t('vscodeExtension.usage.autocompletionTemplates') }}</li>
          <li>{{ $t('vscodeExtension.usage.autocompletionCommands') }}</li>
        </ul>

        <h3>{{ $t('vscodeExtension.usage.validation') }}</h3>
        <p>{{ $t('vscodeExtension.usage.validationDesc') }}</p>
        <ul>
          <li>{{ $t('vscodeExtension.usage.validationDuplicate') }}</li>
          <li>{{ $t('vscodeExtension.usage.validationTypes') }}</li>
          <li>{{ $t('vscodeExtension.usage.validationDirectives') }}</li>
          <li>{{ $t('vscodeExtension.usage.validationIncludes') }}</li>
        </ul>

        <h3>{{ $t('vscodeExtension.usage.navigation') }}</h3>
        <p>{{ $t('vscodeExtension.usage.navigationDesc') }}</p>
        <ul>
          <li><strong>F12</strong> - {{ $t('vscodeExtension.usage.goToDefinition') }}</li>
          <li><strong>Shift+F12</strong> - {{ $t('vscodeExtension.usage.findReferences') }}</li>
          <li><strong>Ctrl+Shift+O</strong> / <strong>Cmd+Shift+O</strong> - {{ $t('vscodeExtension.usage.outline') }}</li>
        </ul>

        <h3>{{ $t('vscodeExtension.usage.commands') }}</h3>
        <p>{{ $t('vscodeExtension.usage.commandsDesc') }}</p>
        <ul>
          <li><strong>{{ $t('vscodeExtension.usage.commandValidate') }}</strong> - {{ $t('vscodeExtension.usage.commandValidateDesc') }}</li>
          <li><strong>{{ $t('vscodeExtension.usage.commandShowAst') }}</strong> - {{ $t('vscodeExtension.usage.commandShowAstDesc') }}</li>
        </ul>
      </section>

      <section id="supported-syntax">
        <h2>{{ $t('vscodeExtension.supportedSyntax.title') }}</h2>
        <p>{{ $t('vscodeExtension.supportedSyntax.desc') }}</p>
        
        <h3>{{ $t('vscodeExtension.supportedSyntax.commands') }}</h3>
        <pre v-pre><code>command_name(param: type = default):
    > desc: Description
    > script: echo "Hello"</code></pre>

        <h3>{{ $t('vscodeExtension.supportedSyntax.directives') }}</h3>
        <p>{{ $t('vscodeExtension.supportedSyntax.directivesDesc') }}</p>
        <ul>
          <li><code>> desc:</code>, <code>> cwd:</code>, <code>> env:</code>, <code>> script:</code></li>
          <li><code>> before:</code>, <code>> after:</code>, <code>> fallback:</code>, <code>> finaly:</code></li>
          <li><code>> depends:</code>, <code>> validate:</code>, <code>> if:</code>, <code>> elif:</code>, <code>> else:</code></li>
          <li><code>> logs:json</code>, <code>> logs:txt</code>, <code>> privileged</code>, <code>> require_confirm:</code></li>
          <li><code>> script[hide]:</code>, <code>> before[hide]:</code>, <code>> after[hide]:</code>, <code>> fallback[hide]:</code>, <code>> finaly[hide]:</code></li>
        </ul>

        <h3>{{ $t('vscodeExtension.supportedSyntax.meta') }}</h3>
        <ul>
          <li><code>@var</code> - {{ $t('vscodeExtension.supportedSyntax.var') }}</li>
          <li><code>@const</code> - {{ $t('vscodeExtension.supportedSyntax.const') }}</li>
          <li><code>@function</code> - {{ $t('vscodeExtension.supportedSyntax.function') }}</li>
          <li><code>@include</code> - {{ $t('vscodeExtension.supportedSyntax.include') }}</li>
        </ul>

        <h3>{{ $t('vscodeExtension.supportedSyntax.templates') }}</h3>
        <p>{{ $t('vscodeExtension.supportedSyntax.templatesDesc') }}</p>
        <ul>
          <li><code>&#123;&#123;variable&#125;&#125;</code> - {{ $t('vscodeExtension.supportedSyntax.templateVar') }}</li>
          <li><code>&#123;&#123;param&#125;&#125;</code> - {{ $t('vscodeExtension.supportedSyntax.templateParam') }}</li>
          <li><code>&#123;&#123;now&#125;&#125;</code>, <code>&#123;&#123;user&#125;&#125;</code> - {{ $t('vscodeExtension.supportedSyntax.templateSpecial') }}</li>
        </ul>
      </section>

      <section id="more-info">
        <h2>{{ $t('vscodeExtension.moreInfo.title') }}</h2>
        <p>{{ $t('vscodeExtension.moreInfo.desc') }}</p>
        <p>
          <a href="https://github.com/quonaro/Nest/tree/main/vscode-nestfile-support" target="_blank">
            {{ $t('vscodeExtension.moreInfo.link') }}
          </a>
        </p>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.vscode-extension {
  padding: 2rem 0;
}

h1 {
  font-size: 3rem;
  font-weight: 700;
  margin-bottom: 2rem;
}

h2 {
  font-size: 2rem;
  font-weight: 700;
  margin-top: 3rem;
  margin-bottom: 1rem;
  padding-top: 2rem;
  border-top: 1px solid var(--color-border);
}

h2:first-of-type {
  border-top: none;
  padding-top: 0;
  margin-top: 0;
}

h3 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-top: 2rem;
  margin-bottom: 1rem;
}

p {
  margin-bottom: 1rem;
  line-height: 1.8;
  color: var(--color-text-light);
}

ul, ol {
  margin: 1rem 0;
  padding-left: 2rem;
  line-height: 1.8;
}

li {
  margin-bottom: 0.5rem;
}

code {
  font-family: var(--font-mono);
  font-size: 0.9em;
  background-color: var(--color-code-bg);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  color: var(--color-code-text);
}

pre {
  background-color: var(--color-code-bg);
  padding: 1rem;
  border-radius: 6px;
  overflow-x: auto;
  margin: 1rem 0;
}

pre code {
  background: none;
  padding: 0;
  color: var(--color-text);
}

a {
  color: var(--color-primary);
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
</style>

