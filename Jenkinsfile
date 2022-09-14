#!groovy

pipeline {

  agent
      {
     label 'benchmark'
     }
   triggers {
        issueCommentTrigger('^\\/run-benchmark.*')
    }
  stages {
    // checking git comment
    stage ('checking git comment') {
         steps {  
             script {
                 deleteDir()
             checkout scm
             env.GIT_COMMENT = sh(script:'git log -1 HEAD --pretty=format:%s', returnStdout: true).trim()

}
}
}
    stage('node rust config') {
      when { expression { return env.GIT_COMMENT == 'run-benchmark'} }
      steps {
        deleteDir()
        checkout scm
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly && make benchmarks'
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
}

