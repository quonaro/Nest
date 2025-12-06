<template>
  <Layout>
    <div class="getting-started">
      <h1>{{ $t('gettingStarted.title') }}</h1>
      
      <section id="installation">
        <h2>{{ $t('gettingStarted.installation') }}</h2>
        <p>
          {{ $t('gettingStarted.installationDesc') }}
        </p>
        
        <h3>{{ $t('gettingStarted.usingInstaller') }}</h3>
        <p>{{ $t('gettingStarted.installerWill') }}</p>
        <ul>
          <li>{{ $t('gettingStarted.installerDetect') }}</li>
          <li>{{ $t('gettingStarted.installerDownload') }}</li>
          <li>{{ $t('gettingStarted.installerInstall') }}</li>
          <li>{{ $t('gettingStarted.installerProvide') }}</li>
        </ul>

        <h4>{{ $t('gettingStarted.linuxMacos') }}</h4>
        <pre v-pre><code>curl -fsSL https://raw.githubusercontent.com/quonaro/nest/main/install.sh | bash</code></pre>

        <h4>{{ $t('gettingStarted.windows') }}</h4>
        <pre v-pre><code>irm https://raw.githubusercontent.com/quonaro/nest/main/install.ps1 | iex</code></pre>

        <h3>{{ $t('gettingStarted.manualInstallation') }}</h3>
        <ol>
          <li>{{ $t('gettingStarted.manualStep1') }} <a href="https://github.com/quonaro/nest/releases" target="_blank">Releases</a></li>
          <li>{{ $t('gettingStarted.manualStep2') }}</li>
          <li>{{ $t('gettingStarted.manualStep3') }}</li>
        </ol>

        <h3>{{ $t('gettingStarted.fromSource') }}</h3>
        <pre v-pre><code>git clone https://github.com/quonaro/nest.git
cd nest
cargo build --release
sudo cp target/release/nest /usr/local/bin/</code></pre>
      </section>

      <section id="first-steps">
        <h2>{{ $t('gettingStarted.firstSteps') }}</h2>
        
        <h3>1. {{ $t('gettingStarted.createNestfile') }}</h3>
        <p>{{ $t('gettingStarted.createNestfileDesc') }}</p>
        <pre v-pre><code>greet(name: str, message: str):
    > desc: Greet someone with a message
    > script: echo "Hello {{name}}, {{message}}!"</code></pre>

        <h3>2. {{ $t('gettingStarted.runCommand') }}</h3>
        <pre v-pre><code>$ nest greet "Alice" "welcome!"
Hello Alice, welcome!</code></pre>

        <h3>3. {{ $t('gettingStarted.viewHelp') }}</h3>
        <pre v-pre><code>$ nest --help
$ nest greet --help</code></pre>
      </section>

      <section id="features">
        <h2>{{ $t('gettingStarted.features') }}</h2>
        
        <h3>{{ $t('gettingStarted.currentlyImplemented') }}</h3>
        <ul>
          <li>✅ <strong>{{ $t('gettingStarted.featureCommandStructure') }}</strong></li>
          <li>✅ <strong>{{ $t('gettingStarted.featureParameters') }}</strong></li>
          <li>✅ <strong>{{ $t('gettingStarted.featureDirectives') }}</strong></li>
          <li>✅ <strong>{{ $t('gettingStarted.featureTemplateProcessing') }}</strong></li>
          <li>✅ <strong>{{ $t('gettingStarted.featureCLIFeatures') }}</strong></li>
          <li>✅ <strong>{{ $t('gettingStarted.featureExecution') }}</strong></li>
        </ul>

        <h3>{{ $t('gettingStarted.futurePlans') }}</h3>
        <ul>
          <li>❌ {{ $t('gettingStarted.futureFunctions') }}</li>
          <li>❌ {{ $t('gettingStarted.futureLifecycle') }}</li>
          <li>❌ {{ $t('gettingStarted.futureDependencies') }}</li>
          <li>❌ {{ $t('gettingStarted.futureFunctionCalls') }}</li>
        </ul>
      </section>
    </div>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '../components/Layout.vue'
</script>

<style scoped>
.getting-started {
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

h4 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
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

