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
      agent {
        label 'benchmark'
      }
      steps {
        deleteDir()
        checkout scm
		sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -s /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly'
      }
    }

    // Static Code Analysis
    stage('Static Code Analysis') {
      agent {
        label 'benchmark'
      }
      steps {
        deleteDir()
        checkout scm
        sh 'env'
      }
    }


  }
}
