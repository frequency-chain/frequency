#!groovy

pipeline {

  agent {
        label 'benchmark'
      }
   triggers {
        issueCommentTrigger('^release-benchmark*')
    }
  stages {
    // node configuration
    stage('node rust config') {
      steps {
        deleteDir()
        checkout scm
        sh '''
                      export GIT_COMMENT=$(git log -1 --pretty=%B)
                      case "$GIT_COMMENT" in
                      '^\/runtime-benchmarks*') echo "performing build ..."
                        mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly && make benchmarks'
                                                           ;;
                                          *) echo "no commit message found aborting build"
                                                           ;;
                                        esac
          '''
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
}
