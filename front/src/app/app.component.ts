import { Component, OnInit, NgZone } from '@angular/core';
import { CommonModule } from '@angular/common';
import { PingService } from './services/ping.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule],
  templateUrl:"./app.component.html"
})
export class AppComponent implements OnInit {
  response: any = null;

  constructor(private pingService: PingService, private ngZone: NgZone) { }

  ngOnInit() {
    this.doPing();
  }

  doPing() {
    this.response = null;
    this.pingService.ping().subscribe({
      next: res => this.ngZone.run(() => this.response = res),
      error: err => this.ngZone.run(() => {
        console.error('Erreur complète :', err); // pour debugger
        this.response = 'Erreur : impossible de joindre le serveur → ' + (err.message || JSON.stringify(err));
      })

    });
  }
}
