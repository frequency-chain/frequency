#!groovy

pipeline {

  agent {
        label 'benchmark'
      }

   triggers {
        issueCommentTrigger('.*release-benchmark*')
    }

  stages {

    // node configuration
    stage('node rust config') {
      steps {
        deleteDir()
        checkout scm
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly'
      }
    }

    // perform benchamark testing
    stage('perform benchmark testing') {
      steps {
        deleteDir()
        checkout scm
        sh 'make benchmarks'
      }
    }


  }
}
